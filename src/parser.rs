use crate::{scan, Error, Result, Stmt, Syntax, Tag, Token};

#[cfg(debug_assertions)]
const STACK_SIZE: usize = 1000; // infinite loop protection

#[derive(Debug)]
pub struct Parser<'s, 't> {
    tokens: &'t [Token<'s>], // code
    ast: Vec<Stmt>,          // what we're building
    pos: usize,              // position in tokens vec
    tags: usize,             // open tags

    #[cfg(debug_assertions)]
    peeked: usize, // infinite loop protection hack
}

pub fn parse<'t>(tokens: &'t [Token]) -> Result<Vec<Stmt>> {
    let mut parser = Parser::from(tokens);
    parser.parse()?;
    Ok(parser.ast)
}

impl<'s, 't> Parser<'s, 't> {
    /// Create a `Parser` from a `TokenStream`.
    pub fn from(tokens: &'t [Token<'s>]) -> Parser<'s, 't> {
        Parser {
            tokens,
            ast: vec![],
            tags: 0,
            pos: 0,
            #[cfg(debug_assertions)]
            peeked: 0,
        }
    }

    /// Parse `TokenStream` into `AST`.
    pub fn parse(&mut self) -> Result<()> {
        while !self.peek_eof() {
            let mut block = self.block()?;
            self.ast.append(&mut block);
            match self.peek_kind() {
                Syntax::Dedent | Syntax::Semi => self.skip(),
                _ => {}
            }
        }
        Ok(())
    }

    /// Peek at next `Token`.
    fn peek(&mut self) -> Option<Token> {
        #[cfg(debug_assertions)]
        {
            self.peeked += 1;
            if self.peeked > STACK_SIZE {
                panic!("infinite loop while peek()ing: {:?}", self.tokens.get(0));
            }
        }
        self.tokens.get(self.pos).map(|t| *t)
    }

    /// Peek two ahead.
    fn peek2(&mut self) -> Option<Token> {
        #[cfg(debug_assertions)]
        {
            self.peeked += 1;
            if self.peeked > STACK_SIZE {
                panic!("infinite loop while peek()ing: {:?}", self.tokens.get(0));
            }
        }
        self.tokens.get(self.pos + 1).map(|t| *t)
    }

    /// Get the next token's kind.
    fn peek_kind(&mut self) -> Syntax {
        self.peek().map(|t| t.kind).unwrap_or(Syntax::None)
    }

    /// Check the next token's literal value.
    fn peek_lit(&mut self, lit: &str) -> bool {
        self.peek().filter(|t| t.literal() == lit).is_some()
    }

    /// Check the next token's kind.
    fn peek_is(&mut self, kind: Syntax) -> bool {
        self.peek_kind() == kind
    }

    /// Will self.next() deliver EOF?
    fn peek_eof(&mut self) -> bool {
        self.peek().is_none()
    }

    /// Advance iterator an return next `Token`.
    fn try_next(&mut self) -> Option<Token> {
        if !self.tokens.is_empty() {
            Some(self.next())
        } else {
            None
        }
    }

    /// Move iterator back.
    fn back(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }

    /// Advance iterator an return next `Token`.
    fn next(&mut self) -> Token {
        #[cfg(debug_assertions)]
        {
            self.peeked = 0;
        }
        let pos = self.pos;
        self.pos += 1;
        *self.tokens.get(pos).unwrap()
    }

    /// Skip one token.
    fn skip(&mut self) {
        let _ = self.next();
    }

    /// Skip all tokens of `kind`.
    fn eat(&mut self, kind: Syntax) {
        while self.peek_is(kind) {
            self.skip();
        }
    }

    /// Trigger parse error for next() token.
    fn error<T, S: AsRef<str>>(&mut self, msg: S) -> Result<T> {
        Err(if let Some(got) = self.try_next() {
            Error::new(
                format!("expected {}, got {:?}", msg.as_ref(), got.kind),
                got.pos,
                got.len,
            )
        } else {
            Error::new(format!("expected {}, got EOF", msg.as_ref()), 0, 0)
        })
    }

    /// Consumes and returns the next token if it's of `kind`,
    /// otherwise errors.
    fn expect(&mut self, kind: Syntax) -> Result<Token> {
        if self.peek_kind() == kind {
            Ok(self.next())
        } else {
            self.error(format!("{:?}", kind))
        }
    }

    /// Consumes and returns the next token if it's an Op that matches
    ///the passed literal value.
    fn expect_op(&mut self, lit: &str) -> Result<Token> {
        if self
            .peek()
            .filter(|t| t.kind == Syntax::Op && t.literal() == lit)
            .is_some()
        {
            Ok(self.next())
        } else {
            self.error(format!("op {}", lit))
        }
    }

    /// Parse a number.
    fn number(&mut self) -> Result<Stmt> {
        Ok(Stmt::Number(self.expect(Syntax::Number)?.to_isize()?))
    }

    /// Parse a string.
    fn string(&mut self) -> Result<Stmt> {
        let tok = self.next();
        let is_interpolated = match tok.kind {
            Syntax::String(is) => is,
            _ => return self.error("String"),
        };

        let lit = tok.to_string();
        if is_interpolated && lit.contains('{') {
            let mut parts = vec![];
            let mut idx = 0;
            while let Some(i) = lit[idx..].find('{') {
                // check for escaped \{}
                if i > 0 && lit[idx..].bytes().nth(i - 1).unwrap_or(b'0') == b'\\' {
                    parts.push(Stmt::String(lit[idx..i + idx - 1].into()));
                    parts.push(Stmt::String(lit[idx + i..i + idx + 1].into()));
                    idx += i + 1;
                    continue;
                }

                {
                    let s = &lit[idx..i + idx];
                    if !s.is_empty() {
                        parts.push(Stmt::String(s.into()));
                    }
                }
                idx += i + 1;
                let mut end = idx;
                for (x, b) in lit[idx..].bytes().enumerate() {
                    if b == b'}' {
                        end = idx + x;
                        break;
                    }
                }
                // What! Rust 'lifetime magic.
                let mut exprs = scan(&lit[idx..end])
                    .and_then(|t| parse(&t))
                    .map_err(|mut e| {
                        e.pos += tok.pos + idx - 1; // probably not right yet...
                        e
                    })?;
                parts.append(&mut exprs);
                idx = end + 1;
            }
            if idx < lit.len() {
                parts.push(Stmt::String(lit[idx..].into()));
            }
            if parts.len() == 1 {
                Ok(parts.remove(0))
            } else {
                Ok(Stmt::Call("concat".into(), parts))
            }
        } else {
            Ok(Stmt::String(lit))
        }
    }

    /// Parse a word.
    fn word(&mut self) -> Result<Stmt> {
        let word = self.expect(Syntax::Word)?;
        Ok(match word.literal() {
            "true" | "false" => Stmt::Bool(word.literal() == "true"),
            _ => Stmt::Word(word.to_string()),
        })
    }

    /// Parse a function literal.
    fn func(&mut self) -> Result<Stmt> {
        self.expect(Syntax::LParen)?;
        let mut args = vec![];
        while !self.peek_is(Syntax::RParen) {
            args.push(self.expect(Syntax::Word)?.to_string());
            if self.peek_is(Syntax::Comma) {
                self.next();
            } else {
                break;
            }
        }
        self.expect(Syntax::RParen)?;
        Ok(Stmt::Fn(args, self.block()?))
    }

    /// Parse a code expression.
    fn expr(&mut self) -> Result<Stmt> {
        let left = self.atom()?;

        if self.peek_kind() != Syntax::Op {
            return Ok(left);
        }

        let next = self.peek().unwrap();
        let lit = next.literal();
        match lit {
            ":=" | "=" => {
                let reassign = lit == "=";
                self.skip(); // skip op
                if let Stmt::Word(name) = left {
                    Ok(Stmt::Assign(name, bx!(self.expr()?), reassign))
                } else {
                    self.error("Word")
                }
            }
            "." => {
                // convert word to str, ex: map.key => index(map, "key")
                self.skip();
                let right = self.expr()?;
                if let Stmt::Word(word) = right {
                    Ok(Stmt::Call("index".into(), vec![left, Stmt::String(word)]))
                } else {
                    Ok(Stmt::Call("index".into(), vec![left, right]))
                }
            }
            _ => {
                // check for += and friends
                if !matches!(lit, "==" | "!=")
                    && lit.bytes().last().filter(|b| *b == b'=').is_some()
                {
                    let op = left.to_string();
                    self.skip();
                    Ok(Stmt::Assign(
                        op.clone(),
                        bx!(Stmt::Call(op, vec![left, self.expr()?])),
                        true, // reassignment
                    ))
                } else {
                    Ok(Stmt::Call(
                        self.next().to_string(),
                        vec![left, self.expr()?],
                    ))
                }
            }
        }
    }

    /// Parse an indivisible unit, as the Ancient Greeks would say.
    fn atom(&mut self) -> Result<Stmt> {
        match self.peek_kind() {
            // Literal
            Syntax::String(..) => Ok(self.string()?),
            Syntax::Number => Ok(self.number()?),
            // Tag
            Syntax::LCaret => self.tag(),
            // Sub-expression
            Syntax::LParen => {
                self.skip();
                let expr = self.expr()?;
                self.expect(Syntax::RParen)?;
                Ok(expr)
            }
            // List
            Syntax::LStaple => {
                self.skip();
                self.eat(Syntax::Semi);
                let mut list = vec![];
                while !self.peek_eof() && !self.peek_is(Syntax::RStaple) {
                    self.eat(Syntax::Semi);
                    list.push(self.expr()?);
                    if self.peek_is(Syntax::RStaple) {
                        break;
                    } else if self.peek_is(Syntax::Semi) {
                        self.eat(Syntax::Semi);
                    } else {
                        self.expect(Syntax::Comma)?;
                    }
                }
                self.eat(Syntax::Semi);
                self.expect(Syntax::RStaple)?;
                Ok(Stmt::List(list))
            }
            // Map
            Syntax::LCurly => {
                self.skip();
                self.eat(Syntax::Semi);
                let mut map = vec![];
                while !self.peek_eof() && !self.peek_is(Syntax::RCurly) {
                    self.eat(Syntax::Semi);
                    let key = match self.peek_kind() {
                        Syntax::Word | Syntax::String(..) | Syntax::Number => {
                            self.next().to_string()
                        }
                        _ => return self.error("String key name"),
                    };
                    self.expect(Syntax::Colon)?;
                    self.eat(Syntax::Semi);
                    let val = self.expr()?;
                    map.push((key, val));
                    if self.peek_is(Syntax::Semi) {
                        self.eat(Syntax::Semi);
                    } else if self.peek_is(Syntax::RCurly) {
                        break;
                    } else {
                        self.expect(Syntax::Comma)?;
                    }
                }
                self.eat(Syntax::Semi);
                self.expect(Syntax::RCurly)?;
                Ok(Stmt::Map(map))
            }
            // Variables and function calls
            Syntax::Word => {
                let word = self.word()?;

                // check for "fn()" literal
                if let Stmt::Word(w) = &word {
                    if w == "fn" {
                        return self.func();
                    }
                }

                if !self.peek_is(Syntax::LParen) {
                    return Ok(word);
                } else {
                    self.expect(Syntax::LParen)?;
                    let name = word.to_string();
                    let mut args = vec![];
                    while let Some(tok) = self.peek() {
                        match tok.kind {
                            Syntax::RParen => {
                                self.skip();
                                break;
                            }
                            Syntax::Comma => self.skip(),
                            Syntax::LParen
                            | Syntax::LCurly
                            | Syntax::LStaple
                            | Syntax::String(..)
                            | Syntax::Number
                            | Syntax::Word => {
                                args.push(self.expr()?);
                            }
                            _ => return self.error(")"),
                        }
                    }
                    Ok(Stmt::Call(name, args))
                }
            }
            _ => self.error("Atom"),
        }
    }

    /// Parse a block of code, either:
    /// - to the next Dedent if the next() char is an Indent
    ///   or
    /// - to the next ; if the next() char isn't an Indent
    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut block = vec![];
        let mut indented = false;

        if self.peek_is(Syntax::Indent) {
            self.skip();
            indented = true;
        }

        while !self.peek_eof() {
            match self.peek_kind() {
                // Literal
                Syntax::String(..)
                | Syntax::Number
                | Syntax::LParen
                | Syntax::LStaple
                | Syntax::LCurly => {
                    block.push(self.expr()?);
                }

                // Tag
                Syntax::LCaret => {
                    // Look for </closing> tag and bail if found.
                    if !indented && self.peek2().filter(|p| p.literal() == "/").is_some() {
                        break;
                    }
                    // Otherwise parse as regular tag expression.
                    block.push(self.expr()?);
                }

                // Keyword or Stmtession
                Syntax::Word => {
                    if let Some(word) = self.peek() {
                        match word.literal() {
                            "if" => block.push(self.if_expr()?),
                            "for" => block.push(self.for_expr()?),
                            "def" => block.push(self.def_stmt()?),
                            "return" => {
                                self.skip();
                                block.push(if self.peek_is(Syntax::Semi) {
                                    Stmt::Return(bx!(Stmt::None))
                                } else {
                                    Stmt::Return(bx!(self.expr()?))
                                });
                                self.expect(Syntax::Semi)?;
                            }
                            _ => block.push(self.expr()?),
                        }
                    }
                }

                // keep going if we're indented
                Syntax::Semi if indented => {
                    self.skip();
                }

                // pass these up the food chain
                Syntax::Dedent | Syntax::Semi => break,

                // probably implicit text...
                Syntax::Op => {
                    self.skip();
                    block.push(Stmt::Word(self.next().to_string()));
                }

                // Unexpected
                _ => return self.error("Block stmt"),
            };
        }

        Ok(block)
    }

    /// Parse a `for` statement:
    ///     for v in list
    ///     for k, v in map
    fn for_expr(&mut self) -> Result<Stmt> {
        self.expect(Syntax::Word)?; // for
        let mut key = None;
        let val;

        let word = self.expect(Syntax::Word)?.to_string();
        if self.peek_is(Syntax::Comma) {
            self.skip();
            key = Some(word);
            val = self.next().to_string();
        } else {
            val = word;
        }

        let in_word = self.expect(Syntax::Word)?;
        if in_word.literal() != "in" {
            return self.error("in");
        }

        let iter = self.expr()?;
        let body = self.block()?;

        self.expect(Syntax::Dedent)?;
        Ok(Stmt::For(key, val, bx!(iter), body))
    }

    /// Parse a function definition.
    fn def_stmt(&mut self) -> Result<Stmt> {
        self.expect(Syntax::Word)?; // def
        let name = match self.peek_kind() {
            Syntax::Word | Syntax::Op => self.next(),
            _ => return self.error("function name"),
        }
        .to_string();

        let mut args = vec![];
        self.expect(Syntax::LParen)?;
        while !self.peek_eof() && !self.peek_is(Syntax::RParen) {
            args.push(self.expect(Syntax::Word)?.to_string());
            if self.peek_is(Syntax::Comma) {
                self.next();
            } else {
                break;
            }
        }
        self.expect(Syntax::RParen)?;

        let body = self.block()?;
        Ok(Stmt::Assign(name, bx!(Stmt::Fn(args, body)), false))
    }

    /// Parse an if statement.
    fn if_expr(&mut self) -> Result<Stmt> {
        self.expect(Syntax::Word)?; // if
        let mut conds = vec![];
        let test = self.expr()?;
        let body = self.block()?;
        conds.push((test, body));
        while self.peek_is(Syntax::Dedent) {
            if let Some(next) = self.peek2() {
                if next.literal() == "else" {
                    self.skip(); // skip dedent
                    self.skip(); // skip else
                    let mut test = Stmt::Bool(true);
                    if let Some(word) = self.peek() {
                        if word.literal() == "if" {
                            self.skip();
                            test = self.expr()?;
                        }
                    }
                    let body = if self.peek_is(Syntax::Indent) {
                        self.block()?
                    } else {
                        vec![self.expr()?]
                    };
                    conds.push((test, body));
                    continue;
                }
            }
            break;
        }
        self.expect(Syntax::Dedent)?;
        Ok(Stmt::If(conds))
    }

    /// Parse a <tag> and its contents or a </tag>.
    fn tag(&mut self) -> Result<Stmt> {
        if self.peek2().filter(|p| p.literal() == "/").is_some() {
            self.close_tag()?;
            return Ok(Stmt::None);
        }

        let mut tag = self.open_tag()?;
        if tag.is_closed() {
            return Ok(Stmt::Tag(tag));
        }

        tag.set_body(self.block()?);

        match self.peek_kind() {
            Syntax::Semi | Syntax::None => {
                if self.tags == 0 {
                    self.error("Open Tag")?;
                }
                self.tags -= 1;
            }
            Syntax::Dedent => {
                if self.tags == 0 {
                    self.error("Open Tag")?;
                }
                self.tags -= 1;
                self.skip();
            }
            _ => self.close_tag()?,
        }

        Ok(Stmt::Tag(tag))
    }

    /// Parse just a closing tag, starting after the <
    fn close_tag(&mut self) -> Result<()> {
        if self.tags == 0 {
            return self.error("open tags");
        }
        self.tags -= 1;
        self.expect(Syntax::LCaret)?;
        self.expect_op("/")?;
        // </>
        if self.peek_is(Syntax::RCaret) {
            self.skip();
            return Ok(());
        }
        self.expect(Syntax::String(true))?;
        self.expect(Syntax::RCaret)?;
        Ok(())
    }

    /// Parse a string <opening.tag with=attributes>
    /// starting after the <
    fn open_tag(&mut self) -> Result<Tag> {
        self.tags += 1;
        self.expect(Syntax::LCaret)?;
        let mut tag = Tag::new(match self.peek_kind() {
            Syntax::Op => Stmt::String("div".into()),
            _ => self.attr()?,
        });

        loop {
            let next = self.next();
            let pos = next.pos;
            match next.kind {
                Syntax::Semi => {}
                Syntax::RCaret => break,
                Syntax::Op => match next.literal() {
                    "/" => {
                        tag.close();
                        self.tags -= 1;
                    }
                    "#" => {
                        let id = self.attr()?;
                        if self.peek_lit("=") {
                            self.next();
                            let cond = self.expr()?;
                            tag.set_id(Stmt::Call("when".into(), vec![cond, id]));
                        } else {
                            tag.set_id(id);
                        }
                    }
                    "." => {
                        let class = self.attr()?;
                        if self.peek_lit("=") {
                            self.next();
                            let cond = self.expr()?;
                            tag.add_class(Stmt::Call("when".into(), vec![cond, class]));
                        } else {
                            tag.add_class(class);
                        }
                    }
                    "@" | ":" => {
                        let attr_name = if next.literal() == "@" {
                            Stmt::String("name".into())
                        } else {
                            Stmt::String("type".into())
                        };
                        let expr = self.attr()?;
                        if self.peek_lit("=") {
                            self.next();
                            let cond = self.expr()?;
                            tag.add_attr(attr_name, Stmt::Call("when".into(), vec![cond, expr]));
                        } else {
                            tag.add_attr(attr_name.into(), expr);
                        }
                    }
                    _ => return self.error("# . @ or :"),
                },
                Syntax::String(true) => {
                    self.back();
                    let name = self.attr()?;
                    // single word attributes, like `defer`
                    if !self.peek_lit("=") {
                        tag.add_attr(name, Stmt::Bool(true));
                        continue;
                    }
                    self.expect_op("=")?;
                    match self.peek_kind() {
                        Syntax::Number | Syntax::String(..) | Syntax::Word => {
                            tag.add_attr(name, self.atom()?)
                        }
                        Syntax::JS => tag.add_attr(
                            name,
                            Stmt::String(format!(
                                "(function(e){{ {} }})(event);",
                                self.next().to_string()
                            )),
                        ),

                        _ => return pos_error!(pos, "Expected Word, Number, or String"),
                    }
                }
                _ => return pos_error!(pos, "Expected Attribute or >, got {:?}", next),
            }
        }

        Ok(tag)
    }

    /// Parse a tag attribute, which may have {interpolation}.
    fn attr(&mut self) -> Result<Stmt> {
        self.string()
    }
}
