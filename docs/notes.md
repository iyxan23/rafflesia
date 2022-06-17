random notes idk

### Compile order
 - Read manifest
 - For each activity:
   - Compile the layout and extract information about view IDs
   - Compile the logic code with the provided view IDs (to facilitate GVA)
 
### Full InputType support?
Apparently sketchware just stores bitmasked values of inputType to their raw project files' views lol.
This means we could use any input type we wanted since sketchware doesn't check on it and just straight up applies it
on the resulting xml.