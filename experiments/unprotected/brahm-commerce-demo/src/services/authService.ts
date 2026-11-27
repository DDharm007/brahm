import { randomUUID } from 'node:crypto';
import type { CommerceDatabase } from '../db/database.js';
import { HttpError } from '../middleware/errors.js';
import type { UserRecord, UserRole } from '../models/types.js';
import { hashPassword, verifyPassword } from '../utils/password.js';
import { signSessionToken } from '../utils/token.js';

interface DbUserRow {
  id: string;
  email: string;
  full_name: string;
  password_hash: string;
  role: UserRole;
  created_at: string;
}

export class AuthService {
  constructor(
    private readonly db: CommerceDatabase,
    private readonly tokenSecret: string,
  ) {}

  public registerUser(input: {
    email: string;
    fullName: string;
    password: string;
    role?: UserRole;
  }): { user: UserRecord; token: string } {
    const emailNorm = input.email.trim().toLowerCase();
    const existing = this.db.conn
      .prepare('SELECT id FROM users WHERE email = ?')
      .get(emailNorm) as { id: string } | undefined;
    if (existing) {
      throw new HttpError(409, 'EMAIL_EXISTS', 'Account already exists for this email');
    }

    const userId = `usr_${randomUUID()}`;
    const role: UserRole = input.role ?? 'customer';
    const createdAt = new Date().toISOString();
    const passwordHash = hashPassword(input.password);

    this.db.conn
      .prepare(
        'INSERT INTO users (id, email, full_name, password_hash, role, created_at) VALUES (?, ?, ?, ?, ?, ?)',
      )
      .run(userId, emailNorm, input.fullName.trim(), passwordHash, role, createdAt);

    const signed = signSessionToken(userId, emailNorm, role, this.tokenSecret);
    this.persistSession(userId, signed.tokenHash, signed.claims.exp);

    return {
      user: { id: userId, email: emailNorm, fullName: input.fullName.trim(), role, createdAt },
      token: signed.token,
    };
  }

  public loginUser(email: string, password: string): { user: UserRecord; token: string } {
    const emailNorm = email.trim().toLowerCase();
    const row = this.db.conn
      .prepare(
        'SELECT id, email, full_name, password_hash, role, created_at FROM users WHERE email = ?',
      )
      .get(emailNorm) as DbUserRow | undefined;

    if (!row || !verifyPassword(password, row.password_hash)) {
      throw new HttpError(401, 'INVALID_CREDENTIALS', 'Invalid email or password');
    }

    const signed = signSessionToken(row.id, row.email, row.role, this.tokenSecret);
    this.persistSession(row.id, signed.tokenHash, signed.claims.exp);

    return {
      user: {
        id: row.id,
        email: row.email,
        fullName: row.full_name,
        role: row.role,
        createdAt: row.created_at,
      },
      token: signed.token,
    };
  }

  public getUserById(userId: string): UserRecord {
    const row = this.db.conn
      .prepare('SELECT id, email, full_name, role, created_at FROM users WHERE id = ?')
      .get(userId) as Omit<DbUserRow, 'password_hash'> | undefined;
    if (!row) {
      throw new HttpError(404, 'USER_NOT_FOUND', 'User not found');
    }
    return {
      id: row.id,
      email: row.email,
      fullName: row.full_name,
      role: row.role,
      createdAt: row.created_at,
    };
  }

  private persistSession(userId: string, tokenHash: string, expEpochSec: number): void {
    this.db.conn
      .prepare(
        'INSERT INTO sessions (id, user_id, token_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)',
      )
      .run(
        `ses_${randomUUID()}`,
        userId,
        tokenHash,
        new Date(expEpochSec * 1000).toISOString(),
        new Date().toISOString(),
      );
  }
}
