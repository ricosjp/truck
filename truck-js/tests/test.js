import init, * as Truck from "../pkg/truck_js.js";
import { assertEquals } from "https://deno.land/std@0.110.0/testing/asserts.ts";

Deno.test("tseep cube", async () => {
  await init(Deno.readFile("./pkg/truck_js_bg.wasm"));
  const v = Truck.vertex(-0.5, -0.5, -0.5);
  const e = Truck.tsweep(v.upcast(), [1.0, 0.0, 0.0]);
  const f = Truck.tsweep(e, [0.0, 1.0, 0.0]);
  const abst = Truck.tsweep(f, [0.0, 0.0, 1.0]);
  const solid = abst.into_solid();
  const vec = solid.to_json();
  const readVec = await Deno.readFile("./tests/cube.json");
  assertEquals(vec, readVec);
});

Deno.test("rseep torus", async () => {
  await init(Deno.readFile("./pkg/truck_js_bg.wasm"));
  const v = Truck.vertex(0.5, 0.0, 0.0);
  const w = Truck.rsweep(
    v.upcast(),
    [0.75, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    7.0,
  );
  const abst = Truck.rsweep(
    w,
    [0.0, 0.0, 0.0],
    [0.0, 0.0, 1.0],
    7.0,
  );
  const shell = abst.into_shell();
  const solid = shell.into_solid();
  const vec = solid.to_json();
  const readVec = await Deno.readFile("./tests/torus.json");
  assertEquals(vec, readVec);
});
