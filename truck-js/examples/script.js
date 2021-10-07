import * as Truck from "truck-js";

const cw = 768;
const ch = 768;

var c, gl;

var mouse = [0.0, 0.0];
var rotflag = false;
var camera_position = [0.0, 0.0, 3.0];
var camera_direction = [0.0, 0.0, -1.0];
var camera_updirection = [0.0, 1.0, 0.0];
var camera_gaze = [0.0, 0.0, 0.0];
var fps = 1000 / 30;

var vAttributes;
var vIndex;

var vPositionLocation, vUVLocation, vNormalLocation;

var uniLocation = new Array();

var loaded = true;

const v = Truck.vertex(-0.5, -0.5, -0.5);
const e = Truck.tsweep(v.upcast(), [1.0, 0.0, 0.0]);
const f = Truck.tsweep(e, [0.0, 1.0, 0.0]);
const abst = Truck.tsweep(f, [0.0, 0.0, 1.0]);
const solid = abst.into_solid();
const polygon = solid.to_polygon(0.01);
const object = polygon.to_expanded();
var vBuffer = object.vertex_buffer();
var iBuffer = object.index_buffer();
var index_length = object.indices_length() / 4;

window.onload = function () {
  c = document.getElementById("canvas");
  c.width = cw;
  c.height = ch;

  c.addEventListener("mousemove", mouseMove);
  c.addEventListener("mousedown", mouseDown);
  c.addEventListener("mouseup", mouseUp);

  document.querySelector('input').addEventListener("drop", fileRead);
  document.querySelector('input').addEventListener("change", fileRead);

  gl = c.getContext("webgl2") || c.getContext("experimental-webgl");

  var prg = create_program(
    create_shader("vertexshader"),
    create_shader("fragmentshader"),
  );
  uniLocation[0] = gl.getUniformLocation(prg, "camera_position");
  uniLocation[1] = gl.getUniformLocation(prg, "camera_direction");
  uniLocation[2] = gl.getUniformLocation(prg, "camera_updirection");
  uniLocation[3] = gl.getUniformLocation(prg, "resolution");

  gl.enable(gl.CULL_FACE);
  gl.enable(gl.DEPTH_TEST);

  vPositionLocation = gl.getAttribLocation(prg, "position");
  vUVLocation = gl.getAttribLocation(prg, "uv");
  vNormalLocation = gl.getAttribLocation(prg, "normal");

  gl.clearColor(0.0, 0.0, 0.0, 1.0);
  gl.clearDepth(1.0);

  render();
};

function render() {
  if (loaded) {
    vAttributes = create_vbo(vBuffer);
    vIndex = create_ibo(iBuffer);
    loaded = false;
  }
  
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  gl.uniform3fv(uniLocation[0], camera_position);
  gl.uniform3fv(uniLocation[1], camera_direction);
  gl.uniform3fv(uniLocation[2], camera_updirection);
  gl.uniform2fv(uniLocation[3], [c.width, c.height]);

  gl.bindBuffer(gl.ARRAY_BUFFER, vAttributes);
  gl.enableVertexAttribArray(vPositionLocation);
  gl.vertexAttribPointer(vPositionLocation, 3, gl.FLOAT, false, 3 * 4 + 2 * 4 + 3 * 4, 0);
  gl.enableVertexAttribArray(vUVLocation);
  gl.vertexAttribPointer(vUVLocation, 2, gl.FLOAT, false, 3 * 4 + 2 * 4 + 3 * 4, 3 * 4);
  gl.enableVertexAttribArray(vNormalLocation);
  gl.vertexAttribPointer(vNormalLocation, 3, gl.FLOAT, false, 3 * 4 + 2 * 4 + 3 * 4, 2 * 4 + 3 * 4);

  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, vIndex);

  gl.drawElements(gl.TRIANGLES, index_length, gl.UNSIGNED_SHORT, 0);

  gl.flush();

  setTimeout(render, fps);
}

function rotation(origin, axis, theta, vec) {
  vec = [
    vec[0] - origin[0], vec[1] - origin[1], vec[2] - origin[2]
  ];
  vec = [
    (axis[0] * axis[0] * (1.0 - Math.cos(theta)) + Math.cos(theta)) * vec[0] +
    (axis[0] * axis[1] * (1.0 - Math.cos(theta)) - axis[2] * Math.sin(theta)) *
      vec[1] +
    (axis[0] * axis[2] * (1.0 - Math.cos(theta)) + axis[1] * Math.sin(theta)) *
      vec[2],
    (axis[0] * axis[1] * (1.0 - Math.cos(theta)) + axis[2] * Math.sin(theta)) *
      vec[0] +
    (axis[1] * axis[1] * (1.0 - Math.cos(theta)) + Math.cos(theta)) * vec[1] +
    (axis[1] * axis[2] * (1.0 - Math.cos(theta)) - axis[0] * Math.sin(theta)) *
      vec[2],
    (axis[0] * axis[2] * (1.0 - Math.cos(theta)) - axis[1] * Math.sin(theta)) *
      vec[0] +
    (axis[1] * axis[2] * (1.0 - Math.cos(theta)) + axis[0] * Math.sin(theta)) *
      vec[1] +
    (axis[2] * axis[2] * (1.0 - Math.cos(theta)) + Math.cos(theta)) * vec[2],
  ];
  return [
    vec[0] + origin[0], vec[1] + origin[1], vec[2] + origin[2]
  ];
}

function mouseMove(e) {
  var offset = [e.offsetX, e.offsetY];
  if (rotflag) {
    var diff = [offset[0] - mouse[0], mouse[1] - offset[1]];
    if (diff[0] == 0 || diff[1] == 0) return;
    diff[0] *= 0.01;
    diff[1] *= 0.01;
    var camera_rightdirection = [
      camera_direction[1] * camera_updirection[2] -
      camera_direction[2] * camera_updirection[1],
      camera_direction[2] * camera_updirection[0] -
      camera_direction[0] * camera_updirection[2],
      camera_direction[0] * camera_updirection[1] -
      camera_direction[1] * camera_updirection[0],
    ];
    var axis = [
      diff[0] * camera_updirection[0] - diff[1] * camera_rightdirection[0],
      diff[0] * camera_updirection[1] - diff[1] * camera_rightdirection[1],
      diff[0] * camera_updirection[2] - diff[1] * camera_rightdirection[2],
    ];
    var len = Math.sqrt(
      axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2],
    );
    axis[0] /= len;
    axis[1] /= len;
    axis[2] /= len;
    camera_position = rotation(camera_gaze, axis, -len, camera_position);
    camera_direction = rotation([0.0, 0.0, 0.0], axis, -len, camera_direction);
    camera_updirection = rotation([0.0, 0.0, 0.0], axis, -len, camera_updirection);
  }
  mouse = offset;
}

function mouseDown(_e) {
  rotflag = true;
}

function mouseUp(_e) {
  rotflag = false;
}

function fileRead(e) {
  e.preventDefault();
  const file0 = this.files[0];
  if (typeof file0 === 'undefined') {
    console.log("invalid input");
    return;
  }
  console.log(file0.name);

  var reader = new FileReader();
  reader.readAsArrayBuffer(file0);
  reader.onload = function() {
    const result = new Uint8Array(reader.result);
    const solid = Truck.Solid.from_json(result);
    if (typeof solid === 'undefined') {
      console.log("invalid json");
      return;
    }
    const polygon = solid.to_polygon(0.01);
    if (typeof polygon === 'undefined') {
      console.log("meshing failed");
      return;
    }
    const box = polygon.bounding_box();
    const box_center = [
      (box[0] + box[3]) / 2.0,
      (box[1] + box[4]) / 2.0,
      (box[2] + box[5]) / 2.0
    ];
    camera_position = [
      camera_position[0] - camera_gaze[0] + box_center[0],
      camera_position[1] - camera_gaze[1] + box_center[1],
      camera_position[2] - camera_gaze[2] + box_center[2]
    ];
    camera_gaze = box_center;
    const object = polygon.to_expanded();
    vBuffer = object.vertex_buffer();
    iBuffer = object.index_buffer();
    index_length = object.indices_length() / 4;
    loaded = true;
  };
}

function create_program(vs, fs) {
  var program = gl.createProgram();

  gl.attachShader(program, vs);
  gl.attachShader(program, fs);

  gl.linkProgram(program);

  if (gl.getProgramParameter(program, gl.LINK_STATUS)) {
    gl.useProgram(program);
    return program;
  } else {
    alert(gl.getProgramInfoLog(program));
  }
}

function create_shader(id) {
  var shader;

  var scriptElement = document.getElementById(id);
  if (!scriptElement) return;

  switch (scriptElement.type) {
    case "x-shader/x-vertex":
      shader = gl.createShader(gl.VERTEX_SHADER);
      break;
    case "x-shader/x-fragment":
      shader = gl.createShader(gl.FRAGMENT_SHADER);
      break;
    default:
      return;
  }

  gl.shaderSource(shader, scriptElement.text);
  gl.compileShader(shader);

  if (gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    return shader;
  } else {
    alert(gl.getShaderInfoLog(shader));
  }
}

function create_vbo(data) {
  var vbo = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
  gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
  gl.bindBuffer(gl.ARRAY_BUFFER, null);
  return vbo;
}

function create_ibo(data) {
  var ibo = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, ibo);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(data), gl.STATIC_DRAW);
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
  return ibo;
}
