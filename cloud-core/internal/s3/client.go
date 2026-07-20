// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

// Package s3 integrates with AWS S3 for FITS storage. Key layout:
// docs/storage/s3-layout.md.
package s3

import (
	"context"

	"github.com/aws/aws-sdk-go-v2/service/s3"
)

// Client wraps S3 operations used by Cloud Core.
type Client struct {
	api    *s3.Client
	bucket string
}

func New(api *s3.Client, bucket string) *Client {
	return &Client{api: api, bucket: bucket}
}

// PresignedGet returns a time-limited download URL for a FITS object.
func (c *Client) PresignedGet(ctx context.Context, objectKey string) (string, error) {
	// TODO(Phase 3): build presigned URL via s3.NewPresignClient.
	return "https://" + c.bucket + ".s3.amazonaws.com/" + objectKey, nil
}
