-- Migration 001: Initial Commerce Tables & Seed Categories Support

INSERT OR IGNORE INTO categories (id, slug, name, description) VALUES
  ('cat_hw', 'hardware-keys', 'Hardware Security Keys', 'FIDO2 / WebAuthn physical authenticators'),
  ('cat_bk', 'books', 'Security Engineering Books', 'Cryptography, systems security, and protocol guides'),
  ('cat_cl', 'apparel', 'Engineering Apparel', 'Developer conference gear and accessories');
