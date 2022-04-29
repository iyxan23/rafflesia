# Rafflesia
Rafflesia is a way to write sketchware projects with code. A rafflesia project can be compiled to a sketchware project, and vice versa. It's goal is to make the code transition for sketchware users a little easier as it enforces and teaches common programming practices (and for me to have fun).

Demo:
```
number counter

onCreate {
  display.setTextColor("#000000")
}

button.onClick {
  counter += 1
  display.setText(counter)
}
```

## Rafflesia project
A project is a folder where rafflesia codes resides. It has a configuration file named as `swproj.toml` in the root directory, with its source inside a subdirectory named `src/`. Its source can either be a logic code or a layout code. These codes are then referred on the `swproj.toml` configuration file.

Example project structure:
 - `my-project/`
   - `swproj.toml`
   - `src/`
     - `main.logic`
     - `main.layout`

### `swproj.toml`
`swproj.toml` is a configuration file that stores the metadata and general info of a sketchware project. This file is crucial for identifying a rafflesia project.

Its toml structure is described in this example:
```toml
[project]
id = 600                       # local id, will be ommited on a generated project
name = "My Project"            # the app name
workspace-name = "MyProject"
package = "com.my.project"
version-code = 1
version-name = "1"

time-created = 2022-01-01T00:00:00Z
sw-ver = 150

[project.colors]
primary = "#fff"
primary-dark = "#fff"
accent = "#fff"
control-normal = "#fff"
control-highlight = "#fff"

[activity.main]
logic = "main.logic"      # path relative to src/
layout = "main.layout"

[library.compat]
enabled = true

[library.firebase]
enabled = false
api-key = "AAA"

[library.admob]
enabled = true
test-devices = ["AAAA"]

[library.google-map]
enabled = true
api-key = "AAAA"
```

### `src/*.logic`
Files with a `.logic` extension inside the `src/` folder represents a logic of an activity. This file is then referenced in the `swproj.toml` configuration file.

### `src/*.layout`
Files with a `.layout` extension inside the `src/` folder represents a layout of an activity. Same as logic files, this file is then referenced in the `swproj.toml` configuration file.

## Rafflesia CLI
Rafflesia CLI is a command line app that is used to perform operations on a rafflesia project.

### Commands

#### Creating an empty rafflesia project

Creating an empty rafflesia project will prompt the user into a project configuration state.

```console
$ rafflesia new my-project

## Rafflesia project configuration ##

App name? (My Project) 
Workspace name? (MyProject) 
Package name? (com.user.my.project) 
Version code? (1) 
Version name? (1) 
Use default colors? [yes/no] (yes) 
Enabled libraries? [compat, firebase, admob, googlemap] () 

## Project generated into folder my-project/ ##
## Have fun! ##

$ ls
my-project/
```

#### Generating a rafflesia project from a sketchware project

```console
$ rafflesia generate something.zip

## Project Info ##
# Local ID: 648
# Name: A project
# Package name: com.my.project
# Version code: 14
# Version name: 1.4

## Generated into something/ ##

$ ls
something/
$ ls something/
swproj.toml  src/
```

> It is not planned yet on what sketchware project packaging types will be supported. An idea of mine is to create another library that will support multiple

#### Compiling a rafflesia project into a sketchware project

Compiling a rafflesia project will create a build folder, and will store any cache or intermediary files there. The resulting built file is stored as `build/project-name.zip` (again, it is not planned yet on what sw project packaging types will be supported).

```console
my-project $ rafflesia compile
my-project $ ls
swproj.toml  src/  build/
my-project $ ls build/
my-project.zip  cache/  <coming soon>
```
