# Setting Up HOMEBREW_TAP_TOKEN (Fine-Grained PAT)

The release workflow requires a `HOMEBREW_TAP_TOKEN` secret to automatically update the Homebrew formula. This guide shows how to create a **fine-grained personal access token** with minimal permissions.

## Why Fine-Grained Tokens?

| Token Type | Access Scope | Security Risk |
|------------|--------------|---------------|
| **Fine-grained PAT** ✅ | Only `homebrew-tap` repo | Low - limited blast radius |
| Classic PAT ❌ | ALL your repositories | High - keys to the kingdom |

**Always use fine-grained tokens for automated workflows.**

## Step-by-Step Setup

### Step 1: Create Fine-Grained Personal Access Token

1. Go to **https://github.com/settings/tokens?type=beta**

2. Click **"Generate new token"**

3. Configure the token:

   **Token name:**
   ```
   WebTags Homebrew Formula Updates
   ```

   **Description:** (optional)
   ```
   Allows webtags release workflow to update homebrew-tap formula
   ```

   **Expiration:**
   - Recommended: `90 days` or `1 year`
   - You'll get an email reminder before expiration

   **Repository access:**
   - Select **"Only select repositories"**
   - Click the dropdown and choose: **`adjmunro/homebrew-tap`**
   - ⚠️ DO NOT select "All repositories"

   **Repository permissions:**
   - **Contents**: `Read and write` ✅
   - **Metadata**: `Read-only` (automatic, required)
   - All others: `No access`

4. Scroll to bottom and click **"Generate token"**

5. **⚠️ IMPORTANT**: Copy the token immediately
   - It starts with `github_pat_`
   - You won't be able to see it again
   - Store it temporarily in a password manager

### Step 2: Add Token to Repository Secrets

1. Go to **https://github.com/adjmunro/webtags/settings/secrets/actions**

2. Click **"New repository secret"**

3. Configure the secret:
   - **Name**: `HOMEBREW_TAP_TOKEN`
   - **Secret**: Paste the token (starts with `github_pat_`)

4. Click **"Add secret"**

### Step 3: Verify Setup

✅ You're done! The secret is now available to the release workflow.

To test it works:
- Next release will automatically update the Homebrew formula
- Check the `update-homebrew` job in Actions - it should succeed
- Or wait for the next real release (v0.1.1, etc.)

## Security Best Practices

### ✅ DO:
- Use **fine-grained** tokens (not classic)
- Scope to **minimum repositories** needed
- Set **expiration dates** (force rotation)
- Store tokens in **GitHub Secrets** only
- Rotate tokens when team members leave
- Revoke immediately if compromised

### ❌ DON'T:
- Use classic PATs with `repo` scope
- Give access to all repositories
- Set "No expiration"
- Commit tokens to code
- Share tokens in chat/email
- Reuse tokens across projects

## Troubleshooting

### "Resource not accessible by integration" error
- Token doesn't have `Contents: Write` permission
- Solution: Regenerate token with correct permissions

### "Authentication failed" error
- Token expired or revoked
- Solution: Generate new token and update secret

### "Not found" error
- Token doesn't have access to `homebrew-tap` repository
- Solution: Edit token permissions to include the repository

## Token Rotation

When your token expires (or proactively every 90 days):

1. Generate a new fine-grained token (same steps as above)
2. Update the `HOMEBREW_TAP_TOKEN` secret with new value
3. Revoke the old token at https://github.com/settings/tokens?type=beta

## What This Token Does

The release workflow uses this token to:

1. ✅ Clone `adjmunro/homebrew-tap` repository
2. ✅ Update `Formula/webtags.rb` with new version/SHA256
3. ✅ Commit and push the changes

The token **cannot**:
- ❌ Access any other repositories
- ❌ Modify repository settings
- ❌ Create/delete repositories
- ❌ Access your personal data
- ❌ Perform admin actions

## Alternatives

If you prefer even more security, see:
- `.github/workflows/release-with-deploy-key.yml.example` - SSH deploy key approach
- `.github/workflows/release-with-pr.yml.example` - PR-based manual approval

---

**Status**: After completing these steps, all future releases will automatically update the Homebrew formula with no manual intervention required.
