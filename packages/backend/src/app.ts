import express from "express";
import cors from "cors";
import rateLimit from "express-rate-limit";
import { commitmentRouter } from "./routes/commitment";
import { registerRouter } from "./routes/register";
import { sendRouter } from "./routes/send";

export const app = express();

app.use(cors());
app.use(express.json());

const limiter = rateLimit({
  windowMs: 60 * 1000,
  limit: 30,
  standardHeaders: "draft-7",
  legacyHeaders: false,
  message: { error: "Too many requests, please try again later." },
});
app.use(limiter);

app.get("/health", (_req, res) => res.json({ ok: true }));
app.use("/commitment", commitmentRouter);
app.use("/register", registerRouter);
app.use("/send", sendRouter);
