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
 - [ ] I forgor to add parentheses in the expression 💀
 - [ ] Packaging of project files

[Learn more](docs/rafflesia-overview.md)
