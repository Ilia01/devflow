# DevFlow Roadmap

**Current Version:** 0.2.0
**Last Updated:** 2025-11-02

## What is this?

CLI tool to automate Jira/Git workflows. Stop switching between browser tabs and terminal.

## Version Strategy

- **v0.2.0** (now): Complete all planned features, get team using it
- **v0.3.0** (2-3 weeks): Fix what breaks in real usage
- **v0.5.0** (2-3 months): Production-ready, add CI/CD and binaries
- **v0.9.0** (4-6 months): Public beta, polish UX
- **v1.0.0** (6+ months): Official launch

**Why not rush to 1.0?** Better to get it right than ship it fast. Bad first impression is hard to fix.

**Why no binaries yet?** Team has Rust, `cargo install` is fine. Will add binaries at v0.5 when going public.

---

## v0.2.0 - Complete Phase 1

**Status:** 90% done
**Timeline:** ~1 week

### Done ✅

**Core commands:**
- `devflow init` - Configure Jira, Git, credentials
- `devflow start <ticket>` - Fetch ticket, create branch, update status
- `devflow commit <msg>` - Commit with ticket reference
- `devflow done` - Push, create PR/MR, update Jira
- `devflow status` - Show current ticket/branch

**Discovery:**
- `devflow list` - List assigned tickets
- `devflow search <query>` - Search tickets with filters
- `devflow open [ticket]` - Open ticket/PR in browser

**Config:**
- `devflow config show` - View config (masked secrets)
- `devflow config set <key> <value>` - Update values
- `devflow config validate` - Test API connections
- `devflow config path` - Show config file location

**Enterprise support:**
- PAT auth (Jira Data Center/Server)
- API Token auth (Jira Cloud)
- GitHub + GitLab support
- Jira Data Center compatibility (`/rest/api/latest/`)

**Quality:**
- 46 tests passing
- Custom error types with helpful messages
- Debug mode (`DEVFLOW_DEBUG=1`)
- Secure config (600 permissions)

### TODO 🚧

**Features:**
- [ ] `devflow list --status "In Progress"` - Filter by status
- [ ] `devflow list --project WAB` - Switch projects
- [ ] `devflow list --json` - JSON output for scripting
- [ ] `devflow search --interactive` - Select ticket → auto start work
- [ ] Create `CHANGELOG.md`

**Quality:**
- [ ] Integration test for full workflow (init → start → commit → done)
- [ ] Test error scenarios (network failures, invalid tokens)
- [ ] Better inline help text

**Docs:**
- [ ] Update README with all features
- [ ] Add examples for flags

### Release Checklist

- [ ] All 4 features implemented and tested
- [ ] CHANGELOG.md created
- [ ] All tests passing
- [ ] README updated
- [ ] Merge to main, tag v0.2.0
- [ ] Install on 3-5 teammates' machines
- [ ] Gather feedback

**Install:** `cargo install --path .`

---

## v0.3.0 - Fix What Breaks

**Status:** Not started
**Timeline:** 2-3 weeks after v0.2
**Goal:** Fix everything that annoys teammates in daily use

### Likely Issues

**Performance:**
- [ ] Slow API calls → Add spinners/progress bars
- [ ] Repeated API calls → Cache Jira responses (5-min TTL)
- [ ] Large ticket lists → Pagination

**Edge cases:**
- [ ] Weird ticket formats (special chars, emoji)
- [ ] Network timeouts → Retry logic
- [ ] Very long summaries → Truncate branch names

**UX:**
- [ ] Better error messages (from real usage)
- [ ] Clearer success/failure feedback

**Workflow gaps** (based on team requests):
- [ ] `devflow sync` - Update branch with main/master
- [ ] `devflow abort` - Cancel work, delete branch, return to main
- [ ] `devflow config doctor` - Diagnose setup issues
- [ ] `devflow switch <ticket>` - Switch to different ticket

### Quality

- [ ] Test new features
- [ ] Test edge cases found in production
- [ ] Maintain 50%+ coverage
- [ ] Write errors to `~/.devflow/devflow.log`
- [ ] Add `--verbose` for all commands

### Success Criteria

- [ ] Zero critical bugs from team
- [ ] All teammates using it daily
- [ ] Saves 5+ minutes per ticket workflow
- [ ] All major pain points addressed

**Install:** Still `cargo install --path .`

---

## v0.5.0 - Production Ready

**Status:** Not started
**Timeline:** 1-2 months after v0.3
**Goal:** Zero bugs, ready for public, professional distribution

### Quality

**Testing:**
- [ ] 70%+ test coverage
- [ ] Integration tests with real APIs (opt-in via env var)
- [ ] Cross-platform testing (macOS, Linux)
- [ ] Performance benchmarks (all commands < 2s)
- [ ] Security audit of credential storage

**CI/CD:**
- [ ] GitHub Actions - run tests on every PR
- [ ] Lint with `cargo clippy`
- [ ] Format check with `cargo fmt`
- [ ] Test on macOS + Linux
- [ ] Auto-build binaries on tag push
- [ ] Auto-create GitHub release

**Error handling:**
- [ ] Every error has recovery suggestion
- [ ] Network errors retry 3x with backoff
- [ ] Config validation before API calls

### Distribution (NOW add binaries)

- [ ] macOS (Intel + ARM)
- [ ] Linux x64
- [ ] Windows (if demand)
- [ ] Auto-build via GitHub Actions
- [ ] Upload to GitHub Releases
- [ ] Publish to crates.io

### Documentation

**User docs:**
- [ ] Comprehensive README
- [ ] Troubleshooting guide
- [ ] FAQ
- [ ] Video tutorial (5 min)

**Developer docs:**
- [ ] `CONTRIBUTING.md`
- [ ] `ARCHITECTURE.md`
- [ ] Code comments for complex logic

**Templates:**
- [ ] Issue template (bug)
- [ ] Issue template (feature)
- [ ] PR template
- [ ] `CODE_OF_CONDUCT.md`

### Success Criteria

- [ ] Zero known critical bugs
- [ ] Tests pass on macOS, Linux
- [ ] 10+ people using it
- [ ] Docs cover all features
- [ ] Clean install works for non-Rust devs
- [ ] CI/CD prevents regressions

**Install:**
```bash
cargo install devflow
# OR
curl -L https://github.com/Ilia01/devflow/releases/download/v0.5.0/devflow-macos -o devflow
chmod +x devflow && sudo mv devflow /usr/local/bin/
```

---

## v0.9.0 - Public Beta

**Status:** Not started
**Timeline:** 2-3 months after v0.5
**Goal:** Polish for 1.0, gather community feedback

### UX

**Terminal:**
- [ ] Beautiful output (colors, tables)
- [ ] Progress bars (indicatif)
- [ ] Spinner animations
- [ ] Success/error symbols (✓, ✗, ⚠)

**Dev experience:**
- [ ] Shell autocomplete (bash, zsh, fish)
- [ ] Man pages
- [ ] Better `devflow help`
- [ ] Sensible defaults

### Advanced Features

**Config:**
- [ ] Multiple profiles (`devflow config switch work/personal`)
- [ ] Import/export config
- [ ] Template configs

**Power user:**
- [ ] Plugin system (`~/.devflow/plugins/`)
- [ ] Aliases
- [ ] Hooks (pre-commit, post-done)
- [ ] Custom JQL searches

**Optional:**
- [ ] `devflow tui` - Interactive dashboard
- [ ] Keyboard navigation
- [ ] Launch actions from menu

### Community

**Beta:**
- [ ] Recruit 20-30 beta testers
- [ ] Private Discord for feedback
- [ ] Weekly feedback sessions
- [ ] Feature voting

**Open source:**
- [ ] "Good first issue" labels
- [ ] Respond to issues < 48h
- [ ] Review PRs < 1 week
- [ ] Recognize contributors in CHANGELOG

**Content:**
- [ ] Blog: "Building DevFlow in Rust"
- [ ] Technical deep-dives
- [ ] Video tutorials
- [ ] Share on Twitter/LinkedIn

### Distribution

- [ ] Homebrew: `brew install devflow`
- [ ] Linux packages (apt, yum)
- [ ] Windows (Chocolatey if demand)
- [ ] Docker image

### Success Criteria

- [ ] 50+ active users
- [ ] 20+ beta testers
- [ ] All major features done
- [ ] Zero critical bugs for 2+ weeks
- [ ] Ready for launch

---

## v1.0.0 - Launch

**Status:** Not started
**Timeline:** When ready
**Goal:** Production tool that people actually want to use

### Checklist

**Quality:**
- [ ] All v0.9 criteria met
- [ ] 80%+ test coverage
- [ ] Zero critical bugs
- [ ] Security review done
- [ ] Performance validated (handles 1000+ tickets)
- [ ] Works on macOS, Linux, Windows

**Docs:**
- [ ] Perfect README
- [ ] Comprehensive troubleshooting
- [ ] Video tutorials
- [ ] Migration guide

**Community:**
- [ ] 30+ beta users (2+ weeks)
- [ ] 3+ external contributors
- [ ] Issue templates working
- [ ] Fast response time (< 48h)

### Marketing

**Content:**
- [ ] Launch blog post
- [ ] Technical series
- [ ] Demo video (3-5 min)
- [ ] Case studies

**Platforms:**
- [ ] Product Hunt
- [ ] Reddit (r/rust, r/devops, r/programming)
- [ ] Hacker News
- [ ] Dev.to
- [ ] LinkedIn
- [ ] Twitter

**Distribution:**
- [ ] crates.io
- [ ] Homebrew tap
- [ ] All platforms
- [ ] Listed in awesome-rust, awesome-cli-apps

### Post-Launch

**First month:**
- [ ] Fix critical bugs immediately
- [ ] Respond to all issues < 24h
- [ ] Ship v1.0.1, v1.0.2 with fixes

**Success metrics:**
- [ ] 1000+ downloads
- [ ] 100+ GitHub stars
- [ ] 5+ teams using in production
- [ ] Featured in newsletters

---

## What to Do NOW

### Next 2 weeks (v0.2)

1. Finish 4 remaining features
2. Get 3-5 teammates using it
3. Watch what breaks
4. Fix top 3 pain points

Real usage > everything else.

**Don't:**
- Add features nobody asked for
- Optimize for problems that don't exist
- Build binaries yet

### Weeks 3-6 (v0.3)

1. Fix everything teammates complained about
2. Add progress spinners if APIs feel slow
3. Cache Jira responses
4. Handle edge cases
5. Add the ONE feature people keep asking for

Reliability > new features.

### Months 2-3 (v0.5)

1. Get test coverage to 70%+
2. Set up CI/CD
3. NOW build binaries
4. Write contributor docs
5. Publish to crates.io

Don't rush to 1.0.

### Months 4-6 (v0.9)

1. Get 20-30 beta testers
2. Polish terminal output
3. Add advanced features
4. Write tutorials
5. Make contributing easy

Don't add every feature request.

### Month 6+ (v1.0)

Ship when people need it, not just when it works.

Focus: "Saves 30 min/day"

Best marketing: users telling friends.

---

## Feature Backlog (Post-v1.0)

Don't build these yet. Add when there's actual demand.

**Workflow:**
- `devflow worklog` - Track time
- `devflow sprint` - Sprint planning
- `devflow link <ticket>` - Link branch to different ticket
- `devflow pr review` - PR review workflow
- `devflow rebase` - Smart rebase

**Integrations:**
- Bitbucket
- Azure DevOps
- Slack notifications
- Webhooks

**Advanced:**
- Team analytics
- Custom workflows (YAML)
- API for integrations
- VS Code extension

**QoL:**
- Command aliases
- Commit templates
- Activity history
- Undo operations
- Auto-update notifications

---

## Recent Completions

### Config Management & Jira Data Center (2025-11-02)

**Built:**
1. Config management:
   - `devflow config show/set/validate/path`
   - Config saves even if validation fails
   - Helpful error messages for VPN/network issues

2. Jira Data Center compatibility:
   - Fixed auth for Data Center search endpoint
   - Changed default: `/rest/api/3/` → `/rest/api/latest/`
   - Added `JIRA_API_VERSION` env var override
   - Added `DEVFLOW_DEBUG=1` for logging

**Why it mattered:**
- Users lost config when init validation failed (VPN)
- Data Center `/rest/api/3/search` needs sessions
- `/rest/api/latest/` accepts PAT auth
- Debug mode critical for diagnosing issues

**Files changed:**
- `src/main.rs` - Config commands
- `src/api/jira.rs` - API version, debug logging
- `README.md` - Troubleshooting
- Tests updated to `/rest/api/latest/`

---

## Notes

**Remember:**
- Ship small, iterate fast
- 5 happy users > 50 unused features
- Quality > speed
- Feedback > assumptions
- One perfect workflow > ten half-baked ones

**When to ship:**
- v0.2: Features complete, tests pass
- v0.3: Team uses daily without issues
- v0.5: Strangers can install without help
- v0.9: Beta testers say "this is great"
- v1.0: You'd proudly share on LinkedIn

Build something people love, not just something that works.
