-- Migration: Add embedding columns to artifacts and events (pgvector)
-- embedding_qwen_vl_2b: 2000 dimensions (Qwen VL 2B)
-- embedding_vertex_mm_01: 1408 dimensions (Google Vertex multimodal 001)

CREATE EXTENSION IF NOT EXISTS "vector" WITH SCHEMA "extensions";

ALTER TABLE "public"."artifacts"
  ADD COLUMN "embedding_qwen_vl_2b" "extensions"."vector"(2000),
  ADD COLUMN "embedding_vertex_mm_01" "extensions"."vector"(1408);

ALTER TABLE "public"."events"
  ADD COLUMN "embedding_qwen_vl_2b" "extensions"."vector"(2000),
  ADD COLUMN "embedding_vertex_mm_01" "extensions"."vector"(1408);
