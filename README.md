<h1 align="center"><pre>rafflesia</pre></h1>

A language that compiles to sketchware projects


```txt
number counter

onCreate {
    toast("Hello world")
}

button.onClick {
    counter = counter + 1
    button.setText(counter.toString())
}
```

Status: <b>Very alpha</b>. Project compilation works flawlessly. But there aren't much functions yet, and there are some broken/unimplemented features:
 - [ ] Map and List types (so-called as complex types in the codebase)
 - [ ] Component types
 - [ ] Resources are very much unimplemented
 - [ ] Custom views
 - [ ] I forgor to add parentheses to the grammar 💀
 - [ ] Packaging of project files
 - [ ] Chaining function calls that are separated with newlies doesn't work, because the parser separates statements by newlines. Function chain calls that are separated with newlines will be interpreted as a separate statement and thus will be a parse error.

Cool things to do
 - [ ] A WASM application so users wont need to install rust do unnecessary stuff to test rafflesia.
 - [ ] Direct block coding, so users can manually insert sketchware blocks in their raw form. Just like using asm as in C.
 - [ ] Shadow types for Maps. Since maps aren't typed in sketchware (they're just `HashMap<String, Object>`), it would be nice to have a type-safe map in rafflesia.
 - [ ] Direct java code insertion using something like blockquotes <code>```</code>

[Learn more](docs/rafflesia-overview.md)
