import { Link } from 'react-router-dom'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

function Section({ id, title, children }: { id: string; title: string; children: React.ReactNode }) {
  return (
    <section id={id} className="mb-8">
      <h2 className="text-lg font-bold mb-3 text-foreground">{title}</h2>
      {children}
    </section>
  )
}

function SubSection({ id, title, children }: { id: string; title: string; children: React.ReactNode }) {
  return (
    <div id={id} className="mb-6">
      <h3 className="font-semibold text-sm mb-2 mt-4">{title}</h3>
      {children}
    </div>
  )
}

function Code({ children }: { children: string }) {
  return (
    <pre className="bg-gray-900 text-green-400 text-xs rounded-lg p-4 overflow-x-auto whitespace-pre-wrap break-all">{children}</pre>
  )
}

function ConfigTable({ fields }: { fields: [string, string, string][] }) {
  return (
    <div className="rounded-lg border overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Field</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Description</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {fields.map(([name, type, desc]) => (
            <TableRow key={name}>
              <TableCell className="font-mono text-xs text-primary">{name}</TableCell>
              <TableCell className="text-xs text-muted-foreground">{type}</TableCell>
              <TableCell className="text-xs">{desc}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

function StepNum({ n }: { n: number }) {
  return <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-primary text-primary-foreground text-[10px] font-bold mr-2">{n}</span>
}

const WEBHOOK_PAYLOAD = `{
  "type": "changed",
  "monitor": "API",
  "old_status": "healthy",
  "new_status": "dead",
  "time": "2026-08-08T14:30:00Z"
}`

const SLACK_PAYLOAD = `{
  "text": "<!channel> *API* status changed: \`healthy\` → \`dead\`",
  "attachments": [{
    "fallback": "API status changed",
    "color": "danger",
    "fields": [
      { "title": "Monitor", "value": "API", "short": true },
      { "title": "Status", "value": "healthy → dead", "short": true },
      { "title": "Time", "value": "2026-08-08T14:30:00Z", "short": false }
    ]
  }]
}`

export default function Docs() {
  return (
    <div className="min-h-screen bg-background">
      <header className="border-b border-border bg-card">
        <div className="max-w-3xl mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Link to="/status" className="flex items-center gap-2 hover:opacity-80 no-underline">
              <div className="w-7 h-7 rounded bg-primary flex items-center justify-center text-primary-foreground font-bold text-xs">V</div>
              <span className="font-bold text-foreground">Vigilant</span>
            </Link>
            <span className="text-muted-foreground">/</span>
            <span className="text-foreground font-medium">Docs</span>
          </div>
          <div className="flex gap-3 text-sm">
            <Link to="/status" className="text-muted-foreground hover:text-foreground no-underline">Status</Link>
            <Link to="/login" className="text-muted-foreground hover:text-foreground no-underline">Admin</Link>
          </div>
        </div>
      </header>

      <div className="max-w-3xl mx-auto px-4 py-8">
        <h1 className="text-2xl font-bold mb-2">Vigilant Documentation</h1>
        <p className="text-muted-foreground text-sm mb-8">
          Everything you need to install, configure, and operate Vigilant — the open-source status page.
        </p>

        {/* ---- NAV ---- */}
        <nav className="flex flex-wrap gap-x-2 gap-y-1 mb-8 text-sm">
          <span className="text-muted-foreground text-xs uppercase tracking-wide">Jump to:</span>
          {[
            'quickstart', 'env-vars', 'database', 'admin-overview',
            'admin-monitors', 'admin-notifications', 'admin-users',
            'admin-announcements', 'admin-settings',
            'notif-webhook', 'notif-slack', 'notif-email', 'notif-telegram',
            'notif-twilio', 'notif-pushover', 'notif-gotify',
            'notif-zulip', 'notif-matrix', 'notif-webex',
            'api', 'testing'
          ].map((id, i, arr) => (
            <span key={id}>
              <a href={`#${id}`} className="text-primary hover:underline">{id.replace('notif-', '').replace('admin-', '')}</a>
              {i < arr.length - 1 && <span className="text-muted-foreground"> · </span>}
            </span>
          ))}
        </nav>

        {/* ========== QUICK START ========== */}
        <Section id="quickstart" title="🚀 Quick Start">
          <ol className="text-sm space-y-2 mb-4">
            <li><StepNum n={1} />Install Rust: <a href="https://rustup.rs" className="text-primary hover:underline" target="_blank" rel="noopener">rustup.rs</a></li>
            <li><StepNum n={2} /><Code>{`git clone https://github.com/SamTV12345/Vigilant.git
cd Vigilant
cargo build --release`}</Code></li>
            <li><StepNum n={3} />Create a <code>.env</code> file (or export environment variables):</li>
          </ol>
          <Code>{`# .env — Vigilant configuration
DATABASE_URL=sqlite:vigilant.db?mode=rwc
JWT_SECRET=your-random-secret-here
LISTEN_ADDR=0.0.0.0:8080
ASSETS_PATH=./res/assets/`}</Code>
          <ol className="text-sm space-y-2 mt-4" start={4}>
            <li><StepNum n={4} />Run the server:</li>
          </ol>
          <Code>{`./target/release/vigilant`}</Code>
          <p className="text-xs text-muted-foreground mt-2">
            Open <a href="http://localhost:8080" className="text-primary hover:underline">http://localhost:8080</a> for the public status page,
            or <Link to="/login" className="text-primary hover:underline">/login</Link> for the admin panel.
          </p>
        </Section>

        {/* ========== ENVIRONMENT VARIABLES ========== */}
        <Section id="env-vars" title="⚙️ Environment Variables">
          <p className="text-sm mb-3">
            Vigilant is configured entirely through environment variables. You can set them directly,
            or place them in a <code>.env</code> file in the working directory.
          </p>
          <ConfigTable fields={[
            ['DATABASE_URL', 'string', 'SQLite connection URL. Default: sqlite:vigilant.db?mode=rwc. The file is auto-created if it does not exist.'],
            ['JWT_SECRET', 'string', 'Secret key for signing JWT tokens. Change this in production! Default: change-me-in-production.'],
            ['LISTEN_ADDR', 'string', 'Host and port to bind the HTTP server. Default: 0.0.0.0:8080.'],
            ['ASSETS_PATH', 'string', 'Path to the built frontend assets directory. Default: ./res/assets/.'],
          ]} />
        </Section>

        {/* ========== DATABASE SETUP ========== */}
        <Section id="database" title="🗄️ Database Setup">
          <p className="text-sm mb-4">
            Vigilant uses <strong>SQLite</strong> as its database. The database is
            auto-migrated on startup — no manual schema work needed.
          </p>

          <SubSection id="db-sqlite" title="SQLite (Default)">
            <p className="text-xs text-muted-foreground mb-2">
              SQLite is the default and simplest option. No external server needed — just a file on disk.
              Perfect for small to medium deployments and single-server setups.
            </p>
            <p className="text-xs text-muted-foreground mb-2">Connection URL format:</p>
            <Code>{`DATABASE_URL=sqlite:vigilant.db?mode=rwc`}</Code>
            <p className="text-xs text-muted-foreground mt-2">
              The <code>?mode=rwc</code> flag tells SQLite to create the file if it doesn't exist.
              For an in-memory database (testing only): <code>sqlite::memory:</code>
            </p>
          </SubSection>

          <SubSection id="db-postgres" title="PostgreSQL (Planned)">
            <p className="text-xs text-muted-foreground mb-2">
              PostgreSQL support is planned — the <code>sqlx</code> dependency already includes the
              <code>postgres</code> feature. Once the pool initialization is made driver-generic,
              switching will be as simple as changing the <code>DATABASE_URL</code>.
            </p>
            <p className="text-xs text-muted-foreground mb-2">Target connection URL format:</p>
            <Code>{`DATABASE_URL=postgres://user:password@localhost:5432/vigilant`}</Code>
            <div className="border border-blue-600/50 rounded-lg p-3 bg-blue-600/10 mt-3">
              <p className="text-xs text-blue-300">
                <strong>📋 Status:</strong> Dependencies are in place. The pool init in{' '}
                <code>src/db/mod.rs</code> needs to switch from <code>SqlitePool</code> to a generic{' '}
                <code>Any</code> pool based on the URL scheme. Interested? PRs welcome!
              </p>
            </div>
          </SubSection>

          <SubSection id="db-migrations" title="Migrations">
            <p className="text-xs text-muted-foreground mb-2">
              Vigilant runs embedded SQL migrations on every startup. These are idempotent — safe to run
              multiple times. Migrations live in <code>migrations/</code> and are compiled into the binary.
            </p>
            <ConfigTable fields={[
              ['001_initial.sql', '—', 'Creates all tables: monitors, checks, notifications, incidents, settings, users, announcements, subscribers. Inserts default admin user.'],
              ['002_users_argon2.sql', '—', 'Adds must_change_password column to users table. Replaces bcrypt admin hash with Argon2 hash.'],
            ]} />
          </SubSection>

          <SubSection id="db-backup" title="Backup & Restore">
            <p className="text-xs text-muted-foreground mb-2">SQLite is a single file — back it up while Vigilant is running:</p>
            <Code>{`# Backup (safe while server is running — SQLite handles concurrent reads)
cp vigilant.db vigilant.db.backup.$(date +%Y%m%d)

# Restore
cp vigilant.db.backup.20260808 vigilant.db`}</Code>
          </SubSection>
        </Section>

        {/* ========== ADMIN GUIDE ========== */}
        <Section id="admin-overview" title="🛡️ Administrator Guide">
          <p className="text-sm mb-4">
            The admin panel is available at <Link to="/login" className="text-primary hover:underline">/login</Link>.
            After logging in, you can manage monitors, notifications, users, announcements, and settings.
          </p>

          <SubSection id="admin-auth" title="Authentication & Security">
            <p className="text-xs text-muted-foreground mb-2">
              Vigilant uses <strong>JWT tokens</strong> for authentication and <strong>Argon2id</strong> for
              password hashing (memory-hard, resistant to GPU/ASIC attacks).
            </p>
            <ul className="text-sm list-disc pl-5 space-y-1 mb-2">
              <li><strong>Default credentials:</strong> <code>admin</code> / <code>admin</code></li>
              <li><strong>First login:</strong> You will be forced to change the default password immediately.</li>
              <li><strong>Token expiry:</strong> JWT tokens expire after 7 days. You will need to log in again.</li>
              <li><strong>JWT secret:</strong> Set <code>JWT_SECRET</code> to a long random string in production.</li>
            </ul>
            <div className="border border-red-600/50 rounded-lg p-3 bg-red-600/10 mt-3">
              <p className="text-xs text-red-400">
                <strong>🔐 Security:</strong> Always change <code>JWT_SECRET</code> and the default admin password
                before exposing Vigilant to the internet. Use <code>openssl rand -hex 32</code> to generate a secret.
              </p>
            </div>
          </SubSection>
        </Section>

        {/* Admin: Monitors */}
        <Section id="admin-monitors" title="📡 Monitors">
          <p className="text-sm mb-3">
            Monitors are services that Vigilant checks periodically. Each monitor has a type, URL,
            polling interval, and optional HTTP settings.
          </p>

          <SubSection id="admin-monitors-types" title="Monitor Types">
            <ConfigTable fields={[
              ['http', 'TCP/HTTP', 'HTTP probe against a URL. Supports GET/HEAD/POST/PUT/PATCH, custom headers, request body, and response body regex matching.'],
              ['tcp', 'TCP socket', 'Raw TCP connect check against host:port. Fails if connection is refused or times out.'],
              ['icmp', 'ICMP ping', 'Ping probe (requires CAP_NET_RAW on Linux).'],
              ['dns', 'DNS lookup', 'DNS resolution check.'],
              ['script', 'Shell script', 'Executes a shell script. Exit code 0 = healthy, 1 = sick, ≥2 = dead.'],
            ]} />
          </SubSection>

          <SubSection id="admin-monitors-create" title="Creating a Monitor">
            <p className="text-xs text-muted-foreground mb-2">
              Navigate to <Link to="/monitors" className="text-primary hover:underline">Admin → Monitors</Link> and click <strong>Add Monitor</strong>.
            </p>
            <ConfigTable fields={[
              ['name', 'string', 'Human-readable name shown on the status page.'],
              ['type', 'http | tcp | icmp | dns | script', 'Probe type.'],
              ['url', 'string', 'Target URL or host:port. Examples: https://api.example.com/health, tcp://db.internal:5432.'],
              ['interval_secs', 'number', 'Seconds between checks. Default: 60.'],
              ['timeout_secs', 'number', 'Seconds before a probe times out. Default: 10.'],
              ['method', 'string (http only)', 'HTTP method. Default: GET.'],
              ['headers', 'JSON string', 'Custom HTTP headers. Default: {}.'],
              ['body', 'string', 'HTTP request body (POST/PUT/PATCH).'],
              ['script', 'string', 'Shell script source (script type only).'],
            ]} />
          </SubSection>

          <SubSection id="admin-monitors-status" title="Monitor Statuses">
            <p className="text-xs text-muted-foreground mb-2">Each monitor can be in one of these states:</p>
            <div className="flex gap-2 flex-wrap mb-3">
              <Badge className="bg-green-600 text-white border-0">healthy</Badge>
              <Badge className="bg-yellow-600 text-white border-0">sick</Badge>
              <Badge className="bg-partial text-white border-0">partial</Badge>
              <Badge className="bg-destructive text-white border-0">dead</Badge>
            </div>
            <ul className="text-xs text-muted-foreground list-disc pl-5 space-y-1">
              <li><strong>healthy</strong> — Probe succeeded, service is operational.</li>
              <li><strong>sick</strong> — Probe responded but was slow or returned a warning.</li>
              <li><strong>partial</strong> — Some replicas are dead but enough are alive to avoid full outage.</li>
              <li><strong>dead</strong> — Probe failed, service is unreachable.</li>
            </ul>
          </SubSection>
        </Section>

        {/* Admin: Notifications */}
        <Section id="admin-notifications" title="🔔 Notification Channels">
          <p className="text-sm mb-3">
            When a monitor status changes, Vigilant sends alerts to all active notification channels.
            Configure channels in <Link to="/notifications" className="text-primary hover:underline">Admin → Notifications</Link>.
            See the <a href="#notif-webhook" className="text-primary hover:underline">Notification Channel Reference</a> below for per-channel config details.
          </p>
          <ConfigTable fields={[
            ['name', 'string', 'Display name for this channel.'],
            ['type', 'string', 'Channel type: webhook, slack, email, telegram, twilio, pushover, gotify, zulip, matrix, webex.'],
            ['config', 'JSON', 'Channel-specific configuration (see reference below).'],
            ['reminders_only', 'boolean', 'If true, only sends reminder/downtime alerts, not initial status changes.'],
            ['active', 'boolean', 'Enable or disable this channel without deleting it.'],
          ]} />
        </Section>

        {/* Admin: Users */}
        <Section id="admin-users" title="👥 User Management">
          <p className="text-sm mb-3">
            Manage admin users from <Link to="/settings" className="text-primary hover:underline">Admin → Settings</Link>.
            Vigilant has a simple role-less user system: all users are administrators.
          </p>

          <SubSection id="admin-users-add" title="Adding a User">
            <p className="text-xs text-muted-foreground mb-2">
              In Settings → Users, enter a username and password, then click <strong>Add User</strong>.
              New users will be forced to change their password on first login.
            </p>
          </SubSection>

          <SubSection id="admin-users-delete" title="Deleting a User">
            <p className="text-xs text-muted-foreground mb-2">
              Click <strong>Delete</strong> next to a user. You cannot delete the last remaining user —
              Vigilant requires at least one admin account.
            </p>
          </SubSection>

          <SubSection id="admin-users-password" title="Changing Your Password">
            <p className="text-xs text-muted-foreground mb-2">
              Use the <strong>Change Password</strong> form at the top of Settings.
              Enter your username, current password, and new password.
            </p>
          </SubSection>

          <SubSection id="admin-users-hash" title="Password Hashing (Argon2)">
            <p className="text-xs text-muted-foreground mb-2">
              Vigilant uses <strong>Argon2id</strong> with default parameters:
            </p>
            <ul className="text-xs text-muted-foreground list-disc pl-5 space-y-1">
              <li>Memory: 19,456 KiB (~19 MiB)</li>
              <li>Iterations: 2</li>
              <li>Parallelism: 1</li>
              <li>Variant: Argon2id (hybrid, resistant to both side-channel and GPU attacks)</li>
            </ul>
            <p className="text-xs text-muted-foreground mt-2">
              You can pre-compute a hash using the bundled tool:
            </p>
            <Code>{`cargo run --bin hash_tool
# Prompts for password, outputs: $argon2id$v=19$m=19456,t=2,p=1$...`}</Code>
          </SubSection>
        </Section>

        {/* Admin: Announcements */}
        <Section id="admin-announcements" title="📢 Announcements">
          <p className="text-sm mb-3">
            Announcements appear on the public status page to inform users about planned maintenance
            or ongoing incidents.
          </p>
          <p className="text-xs text-muted-foreground mb-2">
            From the dashboard, use the <strong>New Announcement</strong> form. Announcements are
            displayed in reverse chronological order on <Link to="/status" className="text-primary hover:underline">/status</Link>.
          </p>
        </Section>

        {/* Admin: Settings */}
        <Section id="admin-settings" title="🔧 Key-Value Settings">
          <p className="text-sm mb-3">
            Vigilant stores runtime settings as key-value pairs (via <Link to="/settings" className="text-primary hover:underline">Admin → Settings</Link>).
            These are separate from environment variables and can be changed without restarting.
          </p>
          <ConfigTable fields={[
            ['poll_interval', 'number', 'Seconds between probe cycles. Default: 120.'],
            ['poll_retry', 'number', 'Number of retries before marking a node dead. Default: 2.'],
            ['poll_delay_dead', 'number', 'Seconds before a failing node is declared dead. Default: 10.'],
            ['poll_delay_sick', 'number', 'Seconds before a slow node is declared sick. Default: 5.'],
            ['reminder_interval', 'number', 'Seconds between downtime reminder notifications.'],
            ['startup_notification', 'boolean', 'Send a notification on server startup. Default: true.'],
          ]} />
          <p className="text-xs text-muted-foreground mt-2">
            Add any key (even custom ones) and reference them in scripts or external integrations.
          </p>
        </Section>

        {/* ========== NOTIFICATION CHANNEL REFERENCE ========== */}
        <h2 className="text-lg font-bold mb-3 mt-8 text-foreground">📨 Notification Channel Reference</h2>
        <p className="text-muted-foreground text-sm mb-6">
          Each notification channel has its own JSON config schema. The <code>reminders_only</code> flag
          (on the channel, not in the config JSON) controls whether the channel only fires for reminder alerts.
        </p>

        {/* Webhook */}
        <Section id="notif-webhook" title="🌐 Webhook">
          <p className="text-sm mb-3">
            Vigilant POSTs a JSON payload to your URL on every status transition. Use this to integrate with
            Discord, Teams, PagerDuty, Pabbly Connect, or any custom service.
          </p>
          <ConfigTable fields={[
            ['hook_url', 'string', 'Full URL to POST the notification to (e.g. Discord webhook URL).'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Admin → Add Channel → Type: webhook → Config:</p>
          <Code>{`{
  "hook_url": "https://discord.com/api/webhooks/123/abc"
}`}</Code>
          <h4 className="font-semibold text-xs mt-4 mb-1">Delivered payload</h4>
          <Code>{WEBHOOK_PAYLOAD}</Code>
          <h4 className="font-semibold text-xs mt-4 mb-1">Discord</h4>
          <p className="text-xs text-muted-foreground">
            Paste the Discord webhook URL <strong>including</strong> the <code>/slack</code> query parameter
            (e.g. <code>https://discord.com/api/webhooks/…?thread_id=…</code>). Discord accepts raw JSON
            and renders it as a plain embed. For a rich embed, pipe it through a middleware or use Discord's{' '}
            <a href="https://discord.com/developers/docs/resources/webhook#execute-slackcompatible-webhook" target="_blank" rel="noopener" className="text-primary hover:underline">
              Slack-compatible webhook
            </a>{' '}
            with the Slack channel below.
          </p>
          <h4 className="font-semibold text-xs mt-3 mb-1">Microsoft Teams</h4>
          <p className="text-xs text-muted-foreground">
            Use the <a href="https://learn.microsoft.com/en-us/microsoftteams/platform/webhooks-and-connectors/how-to/add-incoming-webhook" target="_blank" rel="noopener" className="text-primary hover:underline">
              Incoming Webhook connector
            </a>. Teams expects a different payload format — route it through Zapier, Pabbly Connect,
            or a lightweight adapter function.
          </p>
        </Section>

        {/* Slack */}
        <Section id="notif-slack" title="💬 Slack">
          <p className="text-sm mb-3">
            Sends a richly formatted message including monitor name, old/new status, and timestamp.
            Supports <code>@channel</code> mentions.
          </p>
          <ol className="text-sm list-decimal pl-5 space-y-1 mb-3">
            <li>Go to <a href="https://api.slack.com/apps" target="_blank" rel="noopener" className="text-primary hover:underline">api.slack.com/apps</a> → Create New App → From Scratch</li>
            <li>Enable <strong>Incoming Webhooks</strong> under Features → Activate → Add New Webhook to Workspace</li>
            <li>Pick the channel where notifications should land</li>
            <li>Copy the webhook URL (starts with <code>https://hooks.slack.com/services/…</code>)</li>
          </ol>
          <ConfigTable fields={[
            ['hook_url', 'string', 'Slack incoming webhook URL from the step above.'],
            ['mention_channel', 'boolean', 'Prefix message with @channel (optional, default false).'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example config:</p>
          <Code>{`{
  "hook_url": "https://hooks.slack.com/services/T00/B00/xxxx",
  "mention_channel": true
}`}</Code>
          <h4 className="font-semibold text-xs mt-4 mb-1">Delivered message shape</h4>
          <Code>{SLACK_PAYLOAD}</Code>
        </Section>

        {/* Email */}
        <Section id="notif-email" title="📧 Email (SMTP)">
          <p className="text-sm mb-3">
            Sends an email via any SMTP relay (Gmail, SendGrid, Mailgun, self-hosted, etc.).
          </p>
          <ConfigTable fields={[
            ['smtp_host', 'string', 'SMTP server hostname (e.g. smtp.gmail.com).'],
            ['smtp_port', 'number', 'SMTP port. Default 587 (STARTTLS). 465 for TLS.'],
            ['smtp_username', 'string', 'SMTP username (empty = no auth).'],
            ['smtp_password', 'string', 'SMTP password or app-specific password.'],
            ['from_email', 'string', 'Sender address (e.g. vigilant@mydomain.com).'],
            ['to_email', 'string', 'Recipient address.'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example: Gmail (use an <a href="https://support.google.com/accounts/answer/185833" target="_blank" rel="noopener" className="text-primary hover:underline">app password</a>):</p>
          <Code>{`{
  "smtp_host": "smtp.gmail.com",
  "smtp_port": 587,
  "smtp_username": "you@gmail.com",
  "smtp_password": "abcd efgh ijkl mnop",
  "from_email": "vigilant@mydomain.com",
  "to_email": "alerts@mydomain.com"
}`}</Code>
        </Section>

        {/* Telegram */}
        <Section id="notif-telegram" title="📱 Telegram">
          <p className="text-sm mb-3">Sends a message to a Telegram chat via a bot.</p>
          <ol className="text-sm list-decimal pl-5 space-y-1 mb-3">
            <li>Chat with <a href="https://t.me/botfather" target="_blank" rel="noopener" className="text-primary hover:underline">@BotFather</a> — <code>/newbot</code> — copy the token</li>
            <li>Start a chat with your bot, then visit <code>https://api.telegram.org/bot&lt;TOKEN&gt;/getUpdates</code> to get the chat ID</li>
          </ol>
          <ConfigTable fields={[
            ['bot_token', 'string', 'Bot token from @BotFather.'],
            ['chat_id', 'string', 'Target chat ID (e.g. -100123456789 for channels).'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example:</p>
          <Code>{`{
  "bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
  "chat_id": "-1001234567890"
}`}</Code>
        </Section>

        {/* Twilio */}
        <Section id="notif-twilio" title="📞 Twilio SMS">
          <p className="text-sm mb-3">Sends an SMS via the Twilio API.</p>
          <ConfigTable fields={[
            ['account_sid', 'string', 'Twilio Account SID from console.'],
            ['auth_token', 'string', 'Twilio Auth Token.'],
            ['from', 'string', 'Twilio phone number (E.164, e.g. +15551234567).'],
            ['to', 'string', 'Recipient phone number.'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example:</p>
          <Code>{`{
  "account_sid": "ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "auth_token": "your_auth_token",
  "from": "+15551234567",
  "to": "+4917612345678"
}`}</Code>
        </Section>

        {/* Pushover */}
        <Section id="notif-pushover" title="🔔 Pushover">
          <p className="text-sm mb-3">Sends a push notification via Pushover.</p>
          <ConfigTable fields={[
            ['user_key', 'string', 'Your Pushover user key.'],
            ['api_token', 'string', 'Application API token (create at pushover.net/apps).'],
            ['device', 'string', 'Optional: specific device name to target.'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example:</p>
          <Code>{`{
  "user_key": "uQiRzpo4DXghDmr9QzzfQu27cmVRsG",
  "api_token": "azGDORePK8gMaC0QOYAMyEEuzJnyUi"
}`}</Code>
        </Section>

        {/* Gotify */}
        <Section id="notif-gotify" title="🔔 Gotify">
          <p className="text-sm mb-3">Sends a push notification to a self-hosted Gotify server.</p>
          <ConfigTable fields={[
            ['server_url', 'string', 'Gotify server URL (e.g. https://gotify.mydomain.com).'],
            ['app_token', 'string', 'Application token from the Gotify web UI.'],
            ['priority', 'number', 'Message priority (0–10). Default 5.'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example:</p>
          <Code>{`{
  "server_url": "https://gotify.mydomain.com",
  "app_token": "A4bXyZ9qW2vR7tY8",
  "priority": 8
}`}</Code>
        </Section>

        {/* Zulip */}
        <Section id="notif-zulip" title="💬 Zulip">
          <p className="text-sm mb-3">Sends a message to a Zulip stream or private conversation.</p>
          <p className="text-xs text-muted-foreground mb-2">
            Create a bot at <strong>Settings → Personal → Bots → Add a new bot</strong> in your Zulip instance.
            Copy the bot email and API key.
          </p>
          <ConfigTable fields={[
            ['bot_email', 'string', 'Bot email (e.g. vigilant-bot@mydomain.zulipchat.com).'],
            ['api_key', 'string', 'Bot API key from Zulip settings.'],
            ['site_url', 'string', 'Zulip server URL (e.g. https://mydomain.zulipchat.com).'],
            ['type', 'string', '"stream" or "private". Default "stream".'],
            ['to', 'string', 'Stream name or recipient email. Default "general".'],
            ['topic', 'string', 'Topic name. Default "Vigilant Alerts".'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example:</p>
          <Code>{`{
  "bot_email": "vigilant-bot@mydomain.zulipchat.com",
  "api_key": "abcd1234efgh5678ijkl",
  "site_url": "https://mydomain.zulipchat.com",
  "type": "stream",
  "to": "alerts",
  "topic": "Monitor Status"
}`}</Code>
        </Section>

        {/* Matrix */}
        <Section id="notif-matrix" title="🔐 Matrix">
          <p className="text-sm mb-3">Sends a message to a Matrix room via the Client-Server API.</p>
          <p className="text-xs text-muted-foreground mb-2">
            Use an <a href="https://element.io" target="_blank" rel="noopener" className="text-primary hover:underline">Element</a>{' '}
            client: Settings → Help & About → Access Token. Copy the token and the room ID
            (Room Info → Advanced → Internal room ID).
          </p>
          <ConfigTable fields={[
            ['homeserver_url', 'string', 'Matrix homeserver URL (e.g. https://matrix.org).'],
            ['access_token', 'string', 'Access token for the bot user.'],
            ['room_id', 'string', 'Room ID starting with ! (e.g. !abc123:matrix.org).'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example:</p>
          <Code>{`{
  "homeserver_url": "https://matrix.org",
  "access_token": "syt_Ym90dXNlcg_ABCDEF123456",
  "room_id": "!abc123def456:matrix.org"
}`}</Code>
        </Section>

        {/* Webex */}
        <Section id="notif-webex" title="🟣 Cisco Webex">
          <p className="text-sm mb-3">Sends a message to a Webex room or direct conversation via a bot.</p>
          <p className="text-xs text-muted-foreground mb-2">
            Create a bot at{' '}
            <a href="https://developer.webex.com/my-apps/new/bot" target="_blank" rel="noopener" className="text-primary hover:underline">
              developer.webex.com
            </a>. Copy the bot token, then add the bot to your target room.
          </p>
          <ConfigTable fields={[
            ['bot_token', 'string', 'Bot access token from Webex developer portal.'],
            ['room_id', 'string', 'Target room ID (preferred over to_person_email).'],
            ['to_person_email', 'string', 'Alternative: direct message to this email.'],
          ]} />
          <p className="text-xs text-muted-foreground mt-3 mb-2">Example:</p>
          <Code>{`{
  "bot_token": "NmU4M2...your-token...ZjY0",
  "room_id": "Y2lzY29zcGFyazovL3VzL1JPT00vOGJm..."
}`}</Code>
        </Section>

        {/* ========== API REFERENCE ========== */}
        <Section id="api" title="🔌 API Reference">
          <p className="text-sm mb-3">
            All admin endpoints require a <code>Authorization: Bearer &lt;token&gt;</code> header.
            Obtain a token via <code>POST /api/auth/login</code>.
          </p>

          <SubSection id="api-auth" title="Auth">
            <ConfigTable fields={[
              ['POST /api/auth/login', 'public', 'Login. Body: {username, password}. Returns: {token, must_change_password}.'],
              ['POST /api/auth/change-password', 'public', 'Change password. Body: {username, current_password, new_password}.'],
            ]} />
          </SubSection>

          <SubSection id="api-monitors" title="Monitors (JWT required)">
            <ConfigTable fields={[
              ['GET /api/admin/monitors', '—', 'List all monitors.'],
              ['POST /api/admin/monitors', '—', 'Create monitor. Body: {name, type, url, interval_secs, timeout_secs, method?, headers?, body?, script?}.'],
              ['PUT /api/admin/monitors/{id}', '—', 'Update monitor. Body: partial monitor fields.'],
              ['DELETE /api/admin/monitors/{id}', '—', 'Delete a monitor and all its check history.'],
            ]} />
          </SubSection>

          <SubSection id="api-public" title="Public Endpoints">
            <ConfigTable fields={[
              ['GET /api/status', '—', 'Public status: {status, monitors: [{id, name, type, status, active}]}.'],
              ['GET /api/monitors/{id}/checks', '—', 'Check history. Query: limit, offset.'],
              ['GET /api/monitors/{id}/uptime', '—', 'Uptime stats. Query: period (hours).'],
              ['GET /api/monitors/{id}/uptime/daily', '—', 'Daily uptime breakdown. Query: days.'],
              ['GET /api/incidents', '—', 'Incident history. Query: limit.'],
              ['GET /api/announcements', '—', 'List announcements.'],
              ['POST /api/subscribe', '—', 'Email subscription. Body: {email}.'],
              ['GET /api/feed/atom', '—', 'Atom feed of incidents.'],
            ]} />
          </SubSection>

          <SubSection id="api-admin" title="Admin Endpoints (JWT required)">
            <ConfigTable fields={[
              ['GET /api/admin/notifications', '—', 'List notification channels.'],
              ['POST /api/admin/notifications', '—', 'Create channel.'],
              ['PUT /api/admin/notifications/{id}', '—', 'Update channel.'],
              ['DELETE /api/admin/notifications/{id}', '—', 'Delete channel.'],
              ['GET /api/admin/settings', '—', 'List all settings.'],
              ['PUT /api/admin/settings', '—', 'Upsert a setting. Body: {key, value}.'],
              ['POST /api/admin/announcements', '—', 'Create announcement. Body: {title, text}.'],
              ['DELETE /api/admin/announcements/{id}', '—', 'Delete announcement.'],
              ['GET /api/admin/users', '—', 'List all users.'],
              ['POST /api/admin/users', '—', 'Create user. Body: {username, password}.'],
              ['DELETE /api/admin/users/{id}', '—', 'Delete user (cannot delete last remaining user).'],
            ]} />
          </SubSection>
        </Section>

        {/* ========== TESTING ========== */}
        <Section id="testing" title="🧪 Testing & Troubleshooting">
          <SubSection id="testing-echo" title="Echo Server">
            <p className="text-xs text-muted-foreground mb-2">
              To verify your webhook config, point it at a request inspector first. Free options:
            </p>
            <ul className="text-sm list-disc pl-5 space-y-1">
              <li><a href="https://webhook.site" target="_blank" rel="noopener" className="text-primary hover:underline">webhook.site</a> — instant unique URL, view all incoming requests</li>
              <li><a href="https://requestbin.com" target="_blank" rel="noopener" className="text-primary hover:underline">requestbin.com</a> — similar, with persistent bins</li>
            </ul>
          </SubSection>

          <SubSection id="testing-trigger" title="Trigger Manually">
            <p className="text-xs text-muted-foreground mb-2">
              Toggle a monitor's active flag off → save → on → save. This forces a status
              re-evaluation on the next probe tick, which triggers notifications if the status differs.
            </p>
          </SubSection>

          <SubSection id="testing-icmp" title="ICMP Probes on Linux">
            <p className="text-xs text-muted-foreground mb-2">
              ICMP probes require raw socket permissions. If ICMP monitors always show as dead:
            </p>
            <Code>{`sudo setcap 'cap_net_raw+ep' /path/to/vigilant`}</Code>
          </SubSection>

          <SubSection id="testing-logs" title="Logs">
            <p className="text-xs text-muted-foreground mb-2">
              Set the log level via environment variable:
            </p>
            <Code>{`RUST_LOG=vigilant=debug ./vigilant`}</Code>
            <p className="text-xs text-muted-foreground mt-2">
              Levels: <code>error</code>, <code>warn</code>, <code>info</code>, <code>debug</code>, <code>trace</code>.
            </p>
          </SubSection>
        </Section>

        {/* Implementation status */}
        <div className="border border-border rounded-lg p-4 bg-card mt-8">
          <h2 className="text-sm font-bold mb-3">Channel Implementation Status</h2>
          <div className="grid grid-cols-2 gap-2 text-sm">
            {[
              'webhook', 'slack', 'email', 'telegram', 'twilio',
              'pushover', 'gotify', 'zulip', 'matrix', 'webex',
            ].map(name => (
              <div key={name} className="flex items-center gap-2">
                <Badge className="bg-green/20 text-green border-green/30 text-[10px] uppercase tracking-wide">live</Badge>
                <span className="text-foreground">{name}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      <footer className="border-t border-border mt-8">
        <div className="max-w-3xl mx-auto px-4 py-6 text-xs text-muted-foreground text-center">
          <a href="https://github.com/SamTV12345/Vigilant" className="text-primary hover:underline">Vigilant</a>
          {' · '}open-source monitoring · Fork of{' '}
          <a href="https://github.com/valeriansaliou/vigil" className="text-primary hover:underline">Vigil</a> by Valerian Saliou
        </div>
      </footer>
    </div>
  )
}
