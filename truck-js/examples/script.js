const cw = 768;
const ch = 512;

var c, gl;

var mouse = [0.0, 0.0];
var rotflag = false;
var camera_position = [0.0, 0.0, 3.0];
var camera_direction = [0.0, 0.0, -1.0];
var camera_updirection = [0.0, 1.0, 0.0];
var fps = 1000 / 30;

var vAttributes;
var vIndex;

var vPositionLocation, vUVLocation, vNormalLocation;

var uniLocation = new Array();

import init, * as Truck from "../pkg/truck_js.js";
init(undefined);
const v = Truck.vertex(-0.5, -0.5, -0.5);
const e = Truck.tsweep(v.upcast(), [1.0, 0.0, 0.0]);
const f = Truck.tsweep(e, [0.0, 1.0, 0.0]);
const abst = Truck.tsweep(f, [0.0, 0.0, 1.0]);
const solid = abst.into_solid();
const polygon = solid.to_polygon(0.01);
const object = polygon.to_expanded();
const vBuffer = object.vertex_buffer();
const iBuffer = object.index_buffer();
const index_length = object.index_length() / 4;

window.onload = function () {
  c = document.getElementById("canvas");
  c.width = cw;
  c.height = ch;

  c.addEventListener("mousemove", mouseMove, true);
  c.addEventListener("mousedown", mouseDown, true);
  c.addEventListener("mouseup", mouseUp, true);

  gl = c.getContext("webgl2") || c.getContext("experimental-webgl");

  var prg = create_program(
    create_shader("vertexshader"),
    create_shader("fragmentshader"),
  );
  uniLocation[0] = gl.getUniformLocation(prg, "camera_position");
  uniLocation[1] = gl.getUniformLocation(prg, "camera_direction");
  uniLocation[2] = gl.getUniformLocation(prg, "camera_updirection");
  uniLocation[3] = gl.getUniformLocation(prg, "resolution");
  uniLocation[4] = gl.getUniformLocation(prg, "drawbuffer");

  gl.enable(gl.CULL_FACE);
  gl.enable(gl.DEPTH_TEST);

  vAttributes = create_vbo(vBuffer);
  vIndex = create_ibo(iBuffer);

  vPositionLocation = gl.getAttribLocation(prg, "position");
  vUVLocation = gl.getAttribLocation(prg, "uv");
  vNormalLocation = gl.getAttribLocation(prg, "normal");

  gl.clearColor(0.0, 0.0, 0.0, 1.0);
  gl.clearDepth(1.0);

  render();
};

function render() {
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  gl.uniform3fv(uniLocation[0], camera_position);
  gl.uniform3fv(uniLocation[1], camera_direction);
  gl.uniform3fv(uniLocation[2], camera_updirection);
  gl.uniform2fv(uniLocation[3], [c.width, c.height]);

  gl.bindBuffer(gl.ARRAY_BUFFER, vAttributes);
  gl.enableVertexAttribArray(vPositionLocation);
  gl.vertexAttribPointer(vPositionLocation, 3, gl.FLOAT, false, 0, 0);
  gl.enableVertexAttribArray(vUVLocation);
  gl.vertexAttribPointer(vUVLocation, 2, gl.FLOAT, false, 0, 3 * 4);
  gl.enableVertexAttribArray(vNormalLocation);
  gl.vertexAttribPointer(vNormalLocation, 3, gl.FLOAT, false, 0, 2 * 4 + 3 * 4);

  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, vIndex);

  gl.drawElements(gl.TRIANGLES, index_length, gl.UNSIGNED_SHORT, 0);

  gl.flush();

  setTimeout(render, fps);
}

function rotation(axis, theta, vec) {
  return [
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
    camera_position = rotation(axis, -len, camera_position);
    camera_direction = rotation(axis, -len, camera_direction);
    camera_updirection = rotation(axis, -len, camera_updirection);
  }
  mouse = offset;
}

function mouseDown(e) {
  rotflag = true;
}

function mouseUp(e) {
  rotflag = false;
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
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(data), gl.STATIC_DRAW);
  gl.bindBuffer(gl.ARRAY_BUFFER, null);
  return vbo;
}

function create_ibo(data) {
  var ibo = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, ibo);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Int16Array(data), gl.STATIC_DRAW);
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
  return ibo;
}

function create_framebuffer(width, height) {
  var frameBuffer = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, frameBuffer);

  var depthRenderBuffer = gl.createRenderbuffer();
  gl.bindRenderbuffer(gl.RENDERBUFFER, depthRenderBuffer);
  gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, width, height);
  gl.framebufferRenderbuffer(
    gl.FRAMEBUFFER,
    gl.DEPTH_ATTACHMENT,
    gl.RENDERBUFFER,
    depthRenderBuffer,
  );

  var fTexture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, fTexture);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    width,
    height,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    null,
  );

  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.framebufferTexture2D(
    gl.FRAMEBUFFER,
    gl.COLOR_ATTACHMENT0,
    gl.TEXTURE_2D,
    fTexture,
    0,
  );

  gl.bindTexture(gl.TEXTURE_2D, null);
  gl.bindRenderbuffer(gl.RENDERBUFFER, null);
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  return { f: frameBuffer, d: depthRenderBuffer, t: fTexture };
}

function create_texture(source, i) {
  var img = new Image();
  img.src = source;
  img.onload = function () {
    vTexture[i] = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, vTexture[i]);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, img);
    gl.generateMipmap(gl.TEXTURE_2D);
    gl.bindTexture(gl.TEXTURE_2D, null);
  };
}
