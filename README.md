# Hatter

_Slightly mad, Imba-inspired, static HTML templates. In Rust._

## TODO

- [x] .hat -> Tokens
- [x] Tokens -> AST
- [x] Produce HTML
- [x] Understand indentation
- [x] Tag shortcuts:
    - [x] #
    - [x] .
    - [x] @
    - [x] :
- [x] Attributes
- [x] Implicit "div"
- [x] (inline js event handlers)
- [ ] <style> tag
- [ ] var value
- [ ] string interpolation
- [ ] shortcut interpolation
    (ex: <div .{name}> -> <div class="dog"> when name="dog")
- [ ] fn call
    - [ ] fn call with args
    - [ ] nested fn call
- [ ] if
- [ ] else
- [ ] for k, v in map
- [ ] for v in list
- [ ] VSCode Extension
- [ ] VSCode + luacheck-style LSP
- [ ] luacheck-style tool