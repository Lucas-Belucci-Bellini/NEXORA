# Images referenced by image-generation issues

This directory is reserved for image files and organized image records associated with the NEXORA asset-generation issues.

## Existing repository images already found

Do not regenerate or duplicate these without first checking their use and provenance:

- `docs/autonomous-reports/asset-batch-20261007/nexora-grass-moss.png`
- `docs/autonomous-reports/asset-batch-20261007/nexora-ore-copper.png`
- `docs/autonomous-reports/asset-batch-20261007/nexora-sand-quartz.png`
- `docs/autonomous-reports/asset-batch-20261007/nexora-soil-dry.png`
- `docs/autonomous-reports/asset-batch-20261007/nexora-stone-ash.png`
- `docs/texture-forge/first-generation-construction-set.png`
- `docs/texture-forge/first-generation-ore-set.png`
- `docs/texture-forge/first-generation-terrain-set-2.png`

These are existing files at their current paths; they have not been moved, because the repository may already reference them.

## Image attachments found in issue comments

The following are image attachments embedded in the comments of [Issue #4](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/4) and [Issue #5](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/5). They are organized here by source. The current GitHub connector could identify the public attachment URLs but could not retrieve their binary bytes to commit them as local PNG files in this operation. Therefore, these entries are source references, **not local image files**. An agent running with repository/network file access should download each attachment, verify its actual format and dimensions, then save it under this directory using the naming convention below.

### Issue #4 — Texture pipeline

- [Attachment 01](https://github.com/user-attachments/assets/a1cd31d4-de81-4891-ae86-1801dc389ce5) — suggested local name: `issue-04-attachment-01` (verify format before adding extension).

### Issue #5 — Stones

- [Attachment 01](https://github.com/user-attachments/assets/4a56ea5e-5617-4f23-999e-959a5bdb7b1c) — suggested local name: `issue-05-attachment-01` (verify format before adding extension).
- [Attachment 02](https://github.com/user-attachments/assets/39d68454-da5a-4e67-af95-72a44c06229a) — suggested local name: `issue-05-attachment-02` (verify format before adding extension).
- [Attachment 03](https://github.com/user-attachments/assets/0fa92f73-0087-4d1f-9f40-c0db62c40baf) — suggested local name: `issue-05-attachment-03` (verify format before adding extension).
- [Attachment 04](https://github.com/user-attachments/assets/ee9e3417-1abc-435b-ba28-fdc17243ce58) — suggested local name: `issue-05-attachment-04` (verify format before adding extension).
- [Attachment 05](https://github.com/user-attachments/assets/7420cc5a-9631-4c11-b2b2-de40fddce2c8) — suggested local name: `issue-05-attachment-05` (verify format before adding extension).
- [Attachment 06](https://github.com/user-attachments/assets/b7b32c26-66c1-4415-b3c6-2b9a4ad65520) — suggested local name: `issue-05-attachment-06` (verify format before adding extension).
- [Attachment 07](https://github.com/user-attachments/assets/ea82b9e4-5113-465b-8948-92a938d540c5) — suggested local name: `issue-05-attachment-07` (verify format before adding extension).
- [Attachment 08](https://github.com/user-attachments/assets/2208795f-a652-4ed4-91b2-fda6da4896cc) — suggested local name: `issue-05-attachment-08` (verify format before adding extension).
- [Attachment 09](https://github.com/user-attachments/assets/33de349a-d682-4434-bb97-9b59458d5dc0) — suggested local name: `issue-05-attachment-09` (verify format before adding extension).
- [Attachment 10](https://github.com/user-attachments/assets/33bbfddf-2c7d-408a-821f-8fda8248b341) — suggested local name: `issue-05-attachment-10` (verify format before adding extension).
- [Attachment 11](https://github.com/user-attachments/assets/7a0255a1-38e4-4906-84c1-90912b892e27) — suggested local name: `issue-05-attachment-11` (verify format before adding extension).
- [Attachment 12](https://github.com/user-attachments/assets/bed5a24f-9d01-4831-9479-7e2d7519436f) — suggested local name: `issue-05-attachment-12` (verify format before adding extension).

## Required agent behavior

1. Read the prompt file for the source issue.
2. Check whether each attachment duplicates an existing repository image before adding it.
3. Download and inspect the original binary; do not rename an image to PNG unless its real format is PNG.
4. Save the file in this folder with its issue and attachment number in the filename.
5. Record the original URL, hash, detected format, dimensions, source issue/comment, and purpose in a manifest.
6. Keep original files unchanged; any cleaned or resized version must be a separate derived asset with provenance.
7. Do not use third-party images as source textures unless the license and usage rights are clear. Issue attachments may be references rather than assets approved for use in the game.
