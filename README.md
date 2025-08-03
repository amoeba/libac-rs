# libac

WIP Rust implementation of various data structures and protocols for Asheron's Call.

## Status

- DATs
  - Read/Write
    - Read:
      - Status: Workable but could the API could be tightened up
      - Details: Supports reading DAT files from the filesystem, HTTP, and from inside a Cloudflrae Worker.
    - Write: No support planned.
  - File Types
    - Textures
      - Status: WIP
      - Detail: Support for reading the more common texture formats and writing as various image formats

## Development

Note that this crate must use the same version of the `worker` crate because of type sharing with libac-rs.
