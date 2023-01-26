## Padma

`padma` is the name of the tool/lib used by rafflesia to generate code that generates sketchware blocks that are
otherwise tedious to write by hand. It is intended to be used both in compile-time (generated through a rust prodecural
macro), and to be able to be loaded at runtime (where users can create custom block definitions and custom functions).
Its code lives within the `padma` directory on the root directory as a rust workspace.

> Padma definitions are called as "definitions" in the face of the user to avoid confusion. This "padma" name is only used
> to refer the definition-generating code in the codebase.

Padma definition files are:

 - *`.blks`*
   Contains spec block definitions. It maps opcodes to their specs. Since sketchware doesn't really are about the spec
   and only cares about the order of arguments to be passed, the spec value can be anything as long as the number of
   arguments matches. Custom `.blks` definitions can be used to translate blocks on different languages, or custom
   block display, or to add missing blocks on newer sketchware versions or modded versions.

 - *`.defs`*
   Contains function definitions that maps to block that were defined previously on `.blks` files. This is basically a
   translator of rafflesia function to regular sketchware blocks (that were previously defined on the blks files).

A quick look at them:
 - *`myblocks.blks`*
   ```text
   [component]doToast: "toast %s"
   [operator]&&(b): "%b and %b"
   [operator]toString(s): "toString %n without decimal"
   [operator]+(d): "%d + %d"
   [operator]stringJoin(s): "%s join %s"
   [operator]stringLength(d): "length of %d"
   [math]random: "pick random %d to %d"
   ```
 - *`myfuncs.defs`*
   ```text
   toast(s) { doToast(@0); }
   random: d = random;
   myFunc(s, s) {
     doToast(stringJoin("First text: ", @0));
     doToast(stringJoin("Second text: ", @1));
   }
   ```
   
Padma's purpose is to serve a bridge between the high-level of functions to the low-level of raw sketchware blocks.

## User

Users have the ability to write and develop their own padma definitions.

### Inside a project

Padma files should be placed on a folder called `custom` in a project directory.

```text
my-project
├ swproj.toml
├ src/
│ ├ main.logic
│ └ main.layout
└ custom/
  ├ myblocks.blks
  └ myfuncs.defs
```

These files are then referenced on the `swproj.toml` file on the array of tables called `definition`.

*`swproj.toml`*
```toml
# ... other stuff

[[definition]]
blocks = ["custom/myblocks.blks"]
definitions = ["custom/myfuncs.defs"]
```

### Layering

Padma blocks/definitions can be layered on top of each other depending on their priorities defined in the project manifest.

Its table would look something like this in an `swproj.toml` file:
```toml
[[definition]]
priority = 600
blocks = ["custom/myblocks.blks"]

[[definition]]
priority = 500
blocks = ["custom/foobar.blks"]
definitions = ["custom/foo.defs", "custom/bar.defs"]
```

Soon enough, after padma has matured a bit, rafflesia will have all of its definitions into padma files that are
generated at compile time using padma's procedural macro. This padma definition will be called as the "standard english
definitions". Its priority will be `900` and will be able to be overridden by other padma definitions.

It should also be possible to disable this standard definitions through the manifest, if somehow the user wants to be a
lot freer:

```toml
[project]
# ...
no_std = true
```

## Development

Padma definitions are the definitions of any functions used in the language. It is basically a dictionary of function
to sketchware blocks.

Padma's purpose is to serve a bridge between the high-level of functions to the low-level of raw sketchware blocks.

Padma definition files will be used in both the compilation and generation process of rafflesia and sketchware projects
respectively.

### Caching

At load, every padma definitions must be cached into memory with their respective lookup tables to prevent slow lookups
during compilation.

Or perhaps it would be better to lazy load definitions that are needed (when other definitions have higher priorities).

### Compilation

In compilation, padma definitions are used as a lookup of raw sketchware blocks from functions that are called in
rafflesia logic files.

When the compiler encounters a function definition, it will search through the padma definitions for a raw block
representation of those functions. For example, a global function named `random(d, d): d` was called, and the compiler
needed to know the blocks that should be placed as that function call.

First, the compiler looks up at the project manifest to see other padma definition files. It should then sort the padma
definitions based off of their priorities, and loops over each definition's starting from the highest priority.

As the compiler reads the padma definition (`.defs`), it will look up for the function signature needed to be generated 
into blocks in that padma definition.

If found, it will then look up for the opcodes used in that function's body through indexed padma `.blks` files.

Finally, the compiler will use the blocks specs defined in the `.blks` files and generate them into the final compiled
project.

### Generation

In generation, padma definitions are used as a pattern lookup of raw sketchware blocks to be transformed into functions
into rafflesia for better readability.

## Spec

There are two files of padma: `.blks` and `.defs` files.

### Blocks

`.blks` files are files that define an opcode's spec and its attributes.

```text
                 - block type
                 |               parameters
                 |             -------  
                 |             |     |  param name
                 |             |     |  ---
                 |             |_    |_ |__
[operator]opcode(d): "my block %s or %s.name"
 -------- ------      ----------------------
 L category    |      L the spec
               |
               L opcode
```

`.defs` files are files that define the definitions that will be used in rafflesia.

```text
// will create a function called `toast` that accepts a string argument on the 0th position
// and generates a `doToast` block with the 0th argument being passed onto doToast's 0th parameter
toast(s) {
  doToast(@0);
}
```

> Block types can be seen on [Iyxan23/sketchware-data:data/block-opcodes.md](https://github.com/Iyxan23/sketchware-data/blob/main/data/block-opcodes.md)