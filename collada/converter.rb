require 'nokogiri'

def run
  doc = File.open('cube-sample.dae') { |f| Nokogiri::XML(f) }
  geometry_node = doc.css('library_geometries')
  mesh_node = geometry_node.css('mesh')

  vertices = mesh_node.css('[id=Cube-mesh-positions-array]')[0].content.split.map(&:to_f)
  p vertices

  normals = mesh_node.css('[id=Cube-mesh-normals-array]')[0].content.split.map(&:to_f)
  p normals

  faces = mesh_node.css('triangles').css('p')[0].content.split.map(&:to_i)

  data = []

  while !faces.empty?
    face = faces.shift(3)

    v = face[0] * 3
    n = face[1] * 3

    # Y-negative
    # Z-up
    data << vertices[v]
    data << -vertices[v+2]
    data << vertices[v+1]

    data << normals[n]
    data << -normals[n+2]
    data << normals[n+1]
  end

  p data

  file = File.open('cube.data', 'wb+')
  file.write(data.pack('e*'))
  file.close

  data
end

run
