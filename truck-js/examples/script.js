import init, * as Truck from "./truck_js.js";
await init();

const cw = 768;
const ch = 768;

let c, gl;

let mouse = [0.0, 0.0];
let rotflag = false;
let cameraPosition = [0.0, 0.0, 3.0];
let cameraDirection = [0.0, 0.0, -1.0];
let cameraUpdirection = [0.0, 1.0, 0.0];
let cameraGaze = [0.0, 0.0, 0.0];
const fps = 1000 / 30;

let vAttributes;
let vIndex;

let vPositionLocation, vUVLocation, vNormalLocation;
let uniLocation;

const v = Truck.vertex(-0.5, -0.5, -0.5);
const e = Truck.tsweep(v.upcast(), [1.0, 0.0, 0.0]);
const f = Truck.tsweep(e, [0.0, 1.0, 0.0]);
const abst = Truck.tsweep(f, [0.0, 0.0, 1.0]);
const solid = abst.into_solid();
let polygon = solid.to_polygon(0.01);
const object = polygon.to_buffer();
let vBuffer = object.vertex_buffer();
let iBuffer = object.index_buffer();
let indexLength = object.index_buffer_size() / 4;

let loaded = true;

if (document.readyState !== 'loading') {
  onLoad();
} else {
  addEventListener('load', onLoad, false);
}

function onLoad () {
  c = document.getElementById("canvas");
  c.width = cw;
  c.height = ch;

  c.addEventListener("mousemove", mouseMove);
  c.addEventListener("mousedown", mouseDown);
  c.addEventListener("mouseup", mouseUp);

  document.querySelector("input").addEventListener("change", fileRead);
  document.getElementById("download-mesh").addEventListener("click", downloadObj);

  gl = c.getContext("webgl2") || c.getContext("experimental-webgl");

  const prg = createProgram(
    createShader("vertexshader"),
    createShader("fragmentshader"),
  );
  uniLocation = [
    gl.getUniformLocation(prg, "camera_position"),
    gl.getUniformLocation(prg, "camera_direction"),
    gl.getUniformLocation(prg, "camera_updirection"),
    gl.getUniformLocation(prg, "resolution"),
  ];

  gl.enable(gl.CULL_FACE);
  gl.enable(gl.DEPTH_TEST);

  vPositionLocation = gl.getAttribLocation(prg, "position");
  vUVLocation = gl.getAttribLocation(prg, "uv");
  vNormalLocation = gl.getAttribLocation(prg, "normal");

  gl.clearColor(0.0, 0.0, 0.0, 1.0);
  gl.clearDepth(1.0);

  render();
}

function render() {
  if (loaded) {
    vAttributes = createVbo(vBuffer);
    vIndex = createIbo(iBuffer);
    loaded = false;
  }

  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  gl.uniform3fv(uniLocation[0], cameraPosition);
  gl.uniform3fv(uniLocation[1], cameraDirection);
  gl.uniform3fv(uniLocation[2], cameraUpdirection);
  gl.uniform2fv(uniLocation[3], [c.width, c.height]);

  gl.bindBuffer(gl.ARRAY_BUFFER, vAttributes);
  gl.enableVertexAttribArray(vPositionLocation);
  gl.vertexAttribPointer(
    vPositionLocation,
    3,
    gl.FLOAT,
    false,
    3 * 4 + 2 * 4 + 3 * 4,
    0,
  );
  gl.enableVertexAttribArray(vUVLocation);
  gl.vertexAttribPointer(
    vUVLocation,
    2,
    gl.FLOAT,
    false,
    3 * 4 + 2 * 4 + 3 * 4,
    3 * 4,
  );
  gl.enableVertexAttribArray(vNormalLocation);
  gl.vertexAttribPointer(
    vNormalLocation,
    3,
    gl.FLOAT,
    false,
    3 * 4 + 2 * 4 + 3 * 4,
    2 * 4 + 3 * 4,
  );

  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, vIndex);

  gl.drawElements(gl.TRIANGLES, indexLength, gl.UNSIGNED_SHORT, 0);

  gl.flush();

  setTimeout(render, fps);
}

function rotation(origin, axis, theta, vec) {
  vec = [
    vec[0] - origin[0],
    vec[1] - origin[1],
    vec[2] - origin[2],
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
    vec[0] + origin[0],
    vec[1] + origin[1],
    vec[2] + origin[2],
  ];
}

function mouseMove(e) {
  const offset = [e.offsetX, e.offsetY];
  if (rotflag) {
    const diff = [offset[0] - mouse[0], mouse[1] - offset[1]];
    if (diff[0] == 0 || diff[1] == 0) return;
    diff[0] *= 0.01;
    diff[1] *= 0.01;
    const cameraRightdirection = [
      cameraDirection[1] * cameraUpdirection[2] -
      cameraDirection[2] * cameraUpdirection[1],
      cameraDirection[2] * cameraUpdirection[0] -
      cameraDirection[0] * cameraUpdirection[2],
      cameraDirection[0] * cameraUpdirection[1] -
      cameraDirection[1] * cameraUpdirection[0],
    ];
    const axis = [
      diff[0] * cameraUpdirection[0] - diff[1] * cameraRightdirection[0],
      diff[0] * cameraUpdirection[1] - diff[1] * cameraRightdirection[1],
      diff[0] * cameraUpdirection[2] - diff[1] * cameraRightdirection[2],
    ];
    const len = Math.sqrt(
      axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2],
    );
    axis[0] /= len;
    axis[1] /= len;
    axis[2] /= len;
    cameraPosition = rotation(cameraGaze, axis, -len, cameraPosition);
    cameraDirection = rotation([0.0, 0.0, 0.0], axis, -len, cameraDirection);
    cameraUpdirection = rotation(
      [0.0, 0.0, 0.0],
      axis,
      -len,
      cameraUpdirection,
    );
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
  if (typeof file0 === "undefined") {
    console.warn("invalid input");
    return;
  }
  console.log(file0.name);

  const reader = new FileReader();
  reader.readAsArrayBuffer(file0);
  reader.onload = function () {
    const result = new Uint8Array(reader.result);
    const solid = Truck.Solid.from_json(result);
    if (typeof solid === "undefined") {
      console.warn("invalid json");
      return;
    }
    polygon = solid.to_polygon(0.01);
    if (typeof polygon === "undefined") {
      console.warn("meshing failed");
      return;
    }
    const box = polygon.bounding_box();
    const boxCenter = [
      (box[0] + box[3]) / 2.0,
      (box[1] + box[4]) / 2.0,
      (box[2] + box[5]) / 2.0,
    ];
    cameraPosition = [
      cameraPosition[0] - cameraGaze[0] + boxCenter[0],
      cameraPosition[1] - cameraGaze[1] + boxCenter[1],
      cameraPosition[2] - cameraGaze[2] + boxCenter[2],
    ];
    cameraGaze = boxCenter;
    const object = polygon.to_buffer();
    vBuffer = object.vertex_buffer();
    iBuffer = object.index_buffer();
    indexLength = object.index_buffer_size() / 4;
    loaded = true;
  };
}

function downloadObj(e) {
  e.preventDefault();
  const obj = polygon.to_obj();
  if (typeof obj === "undefined") {
    console.warn("Failed to generate obj.")
    return;
  }
  const blob = new Blob([(new TextDecoder()).decode(obj)], {type: "text/plain"});
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  document.body.appendChild(a);
  a.download = "meshdata.obj";
  a.href = url;
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

function createProgram(vs, fs) {
  const program = gl.createProgram();

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

function createShader(id) {
  let shader;

  const scriptElement = document.getElementById(id);
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

function createVbo(data) {
  const vbo = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
  gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
  gl.bindBuffer(gl.ARRAY_BUFFER, null);
  return vbo;
}

function createIbo(data) {
  const ibo = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, ibo);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(data), gl.STATIC_DRAW);
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
  return ibo;
}
