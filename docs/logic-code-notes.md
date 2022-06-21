## Random notes
Random notes and future plans about the logic language.

 1. Calling `inst.func()` is equal to `func(inst)` if `func` is `ext`, otherwise it's an error.
 2. Adding `#[ALLOW_DIRECT]` on top of a logic file allows manual control of blocks, you can add whatever block you like
    with whatever parameters it has. Will look similar to this:
    ```text
    #[ALLOW_DIRECT]
    
    onCreate {
        0: addSourceDirectly "add source directly %s.inputOnly" ("// hello asd")
        1: true
        2: if "if %b" ("%1") substack1=3
        3: toast "toast %s" ("Hello world!") color="#ffffff"
    }
    ```
 3. Module system for logic files, imagine importing moreblocks and custom components (when/if it will support sw mod).
 4. Cool macro funsies, I might want to take a similar approach to rust; which is by separating procedural macros into a
    separate module.
 5. listmap aint here
 6. also maps cant have types, perhaps this is an oppurtunity to implement some kind of type safety in rafflesia while
    they get compiled down to the same code
 7. moreblocks wen