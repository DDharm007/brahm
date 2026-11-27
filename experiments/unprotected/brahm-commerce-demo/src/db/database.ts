import fs from 'node:fs';
import path from 'node:path';
import { DatabaseSync } from 'node:sqlite';

export class CommerceDatabase {
  public readonly conn: DatabaseSync;

  constructor(dbPath: string) {
    if (dbPath !== ':memory:') {
      fs.mkdirSync(path.dirname(dbPath), { recursive: true });
    }
    this.conn = new DatabaseSync(dbPath);
    this.conn.exec('PRAGMA foreign_keys = ON;');
  }

  public initializeSchema(schemaSqlPath: string, migrationSqlPath?: string): void {
    const schemaSql = fs.readFileSync(schemaSqlPath, 'utf8');
    this.conn.exec(schemaSql);
    if (migrationSqlPath && fs.existsSync(migrationSqlPath)) {
      const migrationSql = fs.readFileSync(migrationSqlPath, 'utf8');
      this.conn.exec(migrationSql);
    }
  }

  public runInTransaction<T>(fn: () => T): T {
    this.conn.exec('BEGIN IMMEDIATE');
    try {
      const result = fn();
      this.conn.exec('COMMIT');
      return result;
    } catch (err) {
      this.conn.exec('ROLLBACK');
      throw err;
    }
  }

  public close(): void {
    this.conn.close();
  }
}
