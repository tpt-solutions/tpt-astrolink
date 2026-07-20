// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package postgres is the data layer for users, nodes, observations, targets
// and astrometry metadata. Schema: docs/storage/postgres-schema.md.
package postgres

import (
	"context"
	"database/sql"
)

// Store wraps a *sql.DB with typed queries.
type Store struct {
	db *sql.DB
}

func New(db *sql.DB) *Store { return &Store{db: db} }

type Node struct {
	ID       string
	OwnerID  string
	Name     string
	Hardware string
	Status   string
}

func (s *Store) UpsertNode(ctx context.Context, n Node) error {
	_, err := s.db.ExecContext(ctx,
		`INSERT INTO nodes (id, owner_id, name, hardware, status)
		 VALUES ($1,$2,$3,$4,$5)
		 ON CONFLICT (id) DO UPDATE SET status=EXCLUDED.status, name=EXCLUDED.name`,
		n.ID, n.OwnerID, n.Name, n.Hardware, n.Status)
	return err
}

func (s *Store) NodeByID(ctx context.Context, id string) (Node, error) {
	var n Node
	err := s.db.QueryRowContext(ctx,
		`SELECT id, owner_id, name, hardware, status FROM nodes WHERE id=$1`, id).
		Scan(&n.ID, &n.OwnerID, &n.Name, &n.Hardware, &n.Status)
	return n, err
}
