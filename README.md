# Hatter

<img src="./img/rhetoric.jpg" align="right" width="350" alt="The Mad Hatter discussing Hatter" />

> It is practically impossible to teach good programming to students
> that have had a prior exposure to _Hatter_: as potential programmers
> they are mentally mutilated beyond hope of regeneration.
>
> -– Edsger W. Dijkstra, certainly

Hatter is an HTML templating language that can be used server side to
produce static HTML. Its syntax is a cheap knock-off of
[Imba](https://imba.io), without any of the fancy JavaScript parts.

It's like a less powerful, 90s-era PHP. But we're talking PHP/FI, none
of that Easy-Bake Oven PHP3 stuff that you could use to build actual
sites.

If you're feeling adventerous, or mad as a hatter, you can use the
standalone binary to turn templates into HTML files, or include the
zero-dependency Rust library in your (web/cli/?) application.

---

## Hello Hatter

Here's what it looks like:

```html
<nav .webview-app=webview?>
  <a href="/signin"> sign in
  <a href="/signup"> sign up
  <ul> for link in nav-links
    <li.small-link> <a href=link.href> link.text
  <form GET="/search">
    <input@query:text placeholder="Search..." /> <input:submit/>

<#main.markdown-body>
  if logged-in?
    <h1> Welcome back, <span.username> name </>!
  else
    <h1> Nice to, uh, see you. <span.aside> Have we met..?
```

Which, if we're logged in as `The Mad Hatter` and `webview?` is
`false`, will generate this:

```html
<nav>
  <a href='/signin'> sign in </a>
  <a href='/signup'> sign up </a>
  <ul>
    <li class='small-link'> <a href='/hats'> Hats </a> </li>
    <li class='small-link'> <a href='/cards'> Cards </a> </li>
    <li class='small-link'> <a href='/tea'> Tea </a> </li>
  </ul>
  <form method='GET' action='/search'>
    <input name='query' type='text' placeholder='Search...' />
    <input type='submit' />
  </form>
</nav>

<div id='main' class='markdown-body'>
  <h1> Welcome back, <span class='username'> The Mad Hatter </span> !
</div>
```

## Features

- [x] Auto-closing HTML tags and code blocks based on indentation.
- [x] Shorthand for `id`, `class`, `type`, and `name` attributes:
  - `<div#id>`
  - `<div.class1.class2>`
  - `<input@form-field-name>`
  - `<input:text>`
- [ ] Basic types:
  - `bool`, `float`, `string`, `list`, `map`, `fn()`
- [x] for loops over `list` and `map`:
  - `<ul> for page in pages do <li id=page-{page.id}> page.name`
  - `for k, v in some-map do <td> k </> <td> v`
- [x] if/else statements
  - `if logged_in? do <h2> Welcome back!`
- [x] Error-checked assignmnent with `:=` and `=`:
  - `name := 'Bob'`  will error if name **is** already set.
  - `name = 'Bob'`  will error if name **isn't** already set.
- [ ] Dynamic values for regular attributes:
  - `<div page-num=page.id>`
- [ ] Conditionally set attributes or enable shorthand:
  - `<div .logged-in=logged-in?>`
  - `<div data-map=is-map?>`
- [ ] String interpolation:
  - `<span.greeting> "Hey there {name}."`
- [ ] Shorthand interpolation:
  - `<span #page-{page.id} .is-{page.type}> page.title`
- [x] Implicit divs:
  - `<#main>` becomes `<div id='main'>`
- [x] Implicit closing tags:
  - `<i>delicious</>` becomes `<i>delicious</i>`
- [x] Call functions defined in Rust:
  - `<div.name> to-uppercase(name)`
- [x] Easy inline JavaScript:
  - `<li> <a onclick=(alert("Oink!"))> 🐷`
- [x] Add your own operators:
  - `op! ++ append`
- [x] Hatter will add a `<!DOCTYPE>` and wrap everything in `<html>` if
  the first tag in your template is `<head>`.

## Future Features

- Define your own functions with `def name(x Type, b Type)`.
- Define your own tags with `def <tag arg=Type>`.
- Arity checking for functions.
- Type checking for functions.

## TODO

- [ ] `do` syntax
- [ ] error when mixing tabs and spaces
- [ ] `if (1 + 15) > 3`
- [ ] <!DOCTYPE html>
- [ ] <!-- html comments -->
- [ ] Basic types
  - [ ] list
    - [ ] literal syntax -> bytecode
  - [ ] map
    - [ ] literal syntax -> bytecode
  - [ ] fn()
    - [ ] literal syntax -> bytecode
  - [ ] <tag>
    - [ ] literal syntax -> bytecode
- [ ] Dynamic values for regular attributes
  - [ ] attribute=expr
- [ ] Conditionally set attributes or enable shorthand
  - [ ] .class=bool?
  - [ ] #id=bool?
  - [ ] data-id=bool?
- [ ] String interpolation
  - [ ] "hey {friend}"
  - [ ] not for ` or '
- [ ] Shorthand interpolation
  - [ ] .{expr}
  - [ ] .something-{expr}-else
  - [ ] #{expr}
  - [ ] @{expr}
- [ ] := error checking
- [ ] = error checking
- [ ] HTMLized error page
- [ ] def <tag attr=Type>
- [ ] def fn(arg Type, arg Type) Type
- [ ] rewrite lexer based on https://github.com/nathanwhit/minimal-yaml
- [ ] show error location when compiling

### docs

- [ ] design
- [ ] layout
- [ ] generator
- [ ] outline
- [ ] content
- [ ] docs for stdlib

### project

- [ ] bomb-ass test suite
- [ ] stdlib
- [ ] VSCode Extension
- [ ] VSCode + luacheck-style LSP
- [ ] luacheck-style tool

