import request from "supertest";
import { app } from "./app";

describe("GET /health", () => {
  it("returns ok", async () => {
    const res = await request(app).get("/health");
    expect(res.status).toBe(200);
    expect(res.body.ok).toBe(true);
  });
});

describe("GET /commitment/:username", () => {
  it("returns registered:false for a valid username", async () => {
    const res = await request(app).get("/commitment/alice");
    expect(res.status).toBe(200);
    expect(res.body).toMatchObject({ username: "alice", registered: false });
  });

  it("rejects username shorter than 3 chars", async () => {
    const res = await request(app).get("/commitment/ab");
    expect(res.status).toBe(400);
  });

  it("rejects username with special characters", async () => {
    const res = await request(app).get("/commitment/al%20ice");
    expect(res.status).toBe(400);
  });

  it("rejects username longer than 31 chars", async () => {
    const res = await request(app).get("/commitment/" + "a".repeat(32));
    expect(res.status).toBe(400);
  });
});

describe("POST /register", () => {
  const valid = {
    commitment: "a".repeat(64),
    nullifier: "b".repeat(64),
  };

  it("accepts valid hex-32 inputs", async () => {
    const res = await request(app).post("/register").send(valid);
    expect(res.status).toBe(202);
    expect(res.body.status).toBe("pending");
  });

  it("rejects missing commitment", async () => {
    const res = await request(app).post("/register").send({ nullifier: valid.nullifier });
    expect(res.status).toBe(400);
  });

  it("rejects short commitment", async () => {
    const res = await request(app).post("/register").send({ commitment: "abc", nullifier: valid.nullifier });
    expect(res.status).toBe(400);
  });
});

describe("POST /send", () => {
  const valid = {
    recipient_commitment: "a".repeat(64),
    stealth_address: "b".repeat(64),
    proof: "c".repeat(128),
    public_inputs: ["d".repeat(64)],
    nullifier: "e".repeat(64),
    amount: "1000000",
  };

  it("accepts a valid send request", async () => {
    const res = await request(app).post("/send").send(valid);
    expect(res.status).toBe(202);
    expect(res.body.status).toBe("pending");
    expect(res.body.txHash).toBeDefined();
  });

  it("rejects missing recipient_commitment", async () => {
    const { recipient_commitment: _, ...rest } = valid;
    const res = await request(app).post("/send").send(rest);
    expect(res.status).toBe(400);
    expect(res.body.error).toMatch(/recipient_commitment/);
  });

  it("rejects invalid stealth_address", async () => {
    const res = await request(app).post("/send").send({ ...valid, stealth_address: "xyz" });
    expect(res.status).toBe(400);
    expect(res.body.error).toMatch(/stealth_address/);
  });

  it("rejects missing proof", async () => {
    const { proof: _, ...rest } = valid;
    const res = await request(app).post("/send").send(rest);
    expect(res.status).toBe(400);
    expect(res.body.error).toMatch(/proof/);
  });

  it("rejects empty public_inputs", async () => {
    const res = await request(app).post("/send").send({ ...valid, public_inputs: [] });
    expect(res.status).toBe(400);
    expect(res.body.error).toMatch(/public_inputs/);
  });

  it("rejects invalid nullifier", async () => {
    const res = await request(app).post("/send").send({ ...valid, nullifier: "short" });
    expect(res.status).toBe(400);
    expect(res.body.error).toMatch(/nullifier/);
  });

  it("rejects zero amount", async () => {
    const res = await request(app).post("/send").send({ ...valid, amount: "0" });
    expect(res.status).toBe(400);
    expect(res.body.error).toMatch(/amount/);
  });

  it("rejects negative amount", async () => {
    const res = await request(app).post("/send").send({ ...valid, amount: "-100" });
    expect(res.status).toBe(400);
    expect(res.body.error).toMatch(/amount/);
  });
});
