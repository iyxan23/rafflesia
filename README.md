<h1 align=center><pre>rafflesia</pre></h1>

<p align=center>A custom-made language compiler for sketchware projects.</p>

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

<p align=center><a href="https://nurihsanalghifari.my.id/rafflesia">Online playground</a> | <a href="docs/rafflesia-overview.md">Learn more</a></p>

<br/>

<h2>Overview</h2>

The rafflesia project aims to provide an easy and robust language that provides a way to generate and modify sketchware projects outside of your android phone; a project like none-other. With ergonomics and modularity in mind, it's easy to get started programming your first sketchware project without ever touching your phone. It is the first ever language built for sketchware!

You can start by reading through the [docs](docs/rafflesia-overview.md), or you could [create an issue](https://github.com/Iyxan23/rafflesia/issues/new) if you need help!
