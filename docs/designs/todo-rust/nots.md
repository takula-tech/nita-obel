editor daemon:
  - assets formats:
    -  pixel images: *.png, *.jpg, *.jpeg => texture
    -  svg images: *.svg =>  binary representation
    -  font: *.otf, *.ttf
    -  *.html, *.css and *.statechart files
  -  convert the resources to the uniform asset formate.
    -  png and jpeg images will be converted to texture
    -  obj file to material, texture and vertex data
  - calculate entity component diffs 