<h1 align=center><pre>rafflesia</pre></h1>

<p align=center>A language that compiles to sketchware projects</p>

<table align=center>
<tr>
<th>Logic</th>
<th>Layout</th>
</tr>
<tr>
<td>

```
number counter

onCreate {
    toast("Hello world")
}

button.onClick {
    counter = counter + 1
    button.setText(counter.toString())
}
```

</td>
<td>

```text
LinearLayout (
    orientation: vertical,
    layout_width: match_parent,
    layout_height: match_parent,
    gravity: center
) {
    Button (text: "Count"): button
}
```

</td>
</tr>
</table>

<pre align=center>
$ rafflesia build
</pre>

<p align=center><a href="docs/rafflesia-overview.md">Learn more</a></p>
