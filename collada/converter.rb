require 'nokogiri'
require 'pry'

def run
  doc = File.open('collada/cube-sample.dae') { |f| Nokogiri::XML(f) }
  geometry_node = doc.css('library_geometries').first.css('geometry').first
  mesh_node = geometry_node.css('mesh')

  triangles_node = mesh_node.css('triangles').first || mesh_node.css('polylist').first
  input_nodes = triangles_node.css('input')

  input_stride = input_nodes.size

  vertex_data = nil
  vertex_offset = nil

  normal_data = nil
  normal_offset = nil

  input_nodes.each do |input|
    semantic = input.attributes["semantic"].value

    if semantic == "VERTEX"
      vertex_offset = input.attributes["offset"].value.to_i
      source_id = input.attributes["source"].value
      source = mesh_node.css(source_id).first

      if source.name == "vertices"
        next_input = source.css('[semantic=POSITION]').first
        source_id = next_input.attributes["source"].value

        source = mesh_node.css(source_id)
      end

      vertex_data = source.css('float_array').first.content.split.map(&:to_f)
    end

    if semantic == "NORMAL"
      normal_offset = input.attributes["offset"].value.to_i
      source_id = input.attributes["source"].value
      source = mesh_node.css(source_id).first

      normal_data = source.css('float_array').first.content.split.map(&:to_f)
    end
  end

  faces = triangles_node.css('p')[0].content.split.map(&:to_i)

  data = []

  while !faces.empty?
    face = faces.shift(input_stride)

    v = face[vertex_offset] * 3
    n = face[normal_offset] * 3

    # Y-negative
    # Z-up
    data << vertex_data[v]
    data << -vertex_data[v+2]
    data << vertex_data[v+1]

    data << normal_data[n]
    data << -normal_data[n+2]
    data << normal_data[n+1]
  end

  p data

  file = File.open('cube.data', 'wb+')
  file.write(data.pack('e*'))
  file.close

  data
end

run
