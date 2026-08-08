-- ============================================================
-- Vigilant: 90-Day Seed Data
-- Erzeugt 5 Monitore mit ~2160 hourly checks je Monitor
-- (90 Tage × 24h) plus realistische Incidents
-- ============================================================

-- Zuerst 5 Monitore anlegen (IGNORE falls schon vorhanden)
INSERT OR IGNORE INTO monitors (id, name, type, url, interval_secs, timeout_secs, active, current_status) VALUES
  ('api',      'Claude API',           'http', 'https://api.anthropic.com',    60, 10, 1, 'healthy'),
  ('web',      'claude.ai',            'http', 'https://claude.ai',            60, 10, 1, 'healthy'),
  ('console',  'Claude Console',       'http', 'https://console.anthropic.com', 60, 10, 1, 'healthy'),
  ('db',       'PostgreSQL Primary',   'tcp',  'db-primary.internal:5432',    30,  5, 1, 'healthy'),
  ('cdn',      'CDN',                  'http', 'https://cdn.anthropic.com',   120, 15, 1, 'healthy');

-- ============================================================
-- Generator: 2160 Stunden (90 Tage) via Cross-Join statt Rekursion
-- ============================================================
INSERT INTO checks (monitor_id, status, response_time_ms, status_code, error, checked_at)
WITH
  digits(x) AS (
    SELECT 0 UNION SELECT 1 UNION SELECT 2 UNION SELECT 3 UNION SELECT 4
    UNION SELECT 5 UNION SELECT 6 UNION SELECT 7 UNION SELECT 8 UNION SELECT 9
  ),
  hours(offset) AS (
    SELECT ones.x + tens.x*10 + huns.x*100 + thos.x*1000
    FROM digits ones, digits tens, digits huns, (SELECT 0 AS x UNION SELECT 1 UNION SELECT 2) thos
  ),
  monitors AS (
    SELECT 'api'     AS id, 9944 AS threshold, 50  AS min_rt, 200 AS max_rt UNION ALL
    SELECT 'web',      9938,         40,  180 UNION ALL
    SELECT 'console',  9983,         60,  250 UNION ALL
    SELECT 'db',       9850,         5,    50 UNION ALL
    SELECT 'cdn',      10000,        20,  120
  )
SELECT
  m.id,
  CASE
    WHEN abs(random() % 10000) < m.threshold THEN 'healthy'
    WHEN abs(random() % 10) < 6 THEN 'dead'
    ELSE 'sick'
  END,
  CASE
    WHEN abs(random() % 10000) < m.threshold
      THEN abs(random() % (m.max_rt - m.min_rt + 1)) + m.min_rt
    WHEN abs(random() % 10) >= 6
      THEN abs(random() % 1500) + 500
    ELSE NULL
  END,
  CASE
    WHEN abs(random() % 10000) < m.threshold THEN 200
    WHEN abs(random() % 10) >= 6 THEN 502
    ELSE NULL
  END,
  CASE
    WHEN abs(random() % 10000) >= m.threshold AND abs(random() % 10) < 6
      THEN CASE abs(random() % 3)
        WHEN 0 THEN 'Connection refused'
        WHEN 1 THEN 'Timeout after 10s'
        ELSE 'HTTP 500 Internal Server Error'
      END
    WHEN abs(random() % 10000) >= m.threshold
      THEN CASE abs(random() % 3)
        WHEN 0 THEN 'HTTP 503 Service Unavailable'
        WHEN 1 THEN 'Slow response: 2500ms'
        ELSE 'TLS handshake failed'
      END
    ELSE NULL
  END,
  datetime('now', '-' || h.offset || ' hours')
FROM hours h
CROSS JOIN monitors m
WHERE h.offset < 2160;  -- 90 days × 24 hours

-- ============================================================
-- Realistische Incidents (basierend auf den generierten Ausfällen)
-- ============================================================

-- api: 2 Vorfälle in den letzten 90 Tagen
INSERT INTO incidents (id, monitor_id, started_at, resolved_at, status) VALUES
  ('inc-api-1', 'api', datetime('now', '-70 days', '+3 hours'), datetime('now', '-70 days', '+5 hours'), 'resolved'),
  ('inc-api-2', 'api', datetime('now', '-12 days'),           datetime('now', '-12 days', '+1 hours'), 'resolved');

-- web: 1 Vorfall
INSERT INTO incidents (id, monitor_id, started_at, resolved_at, status) VALUES
  ('inc-web-1', 'web', datetime('now', '-45 days', '+8 hours'), datetime('now', '-45 days', '+10 hours'), 'resolved');

-- console: 1 Vorfall
INSERT INTO incidents (id, monitor_id, started_at, resolved_at, status) VALUES
  ('inc-console-1', 'console', datetime('now', '-20 days', '+14 hours'), datetime('now', '-20 days', '+15 hours'), 'resolved');

-- db: 4 Vorfälle (mehr Probleme)
INSERT INTO incidents (id, monitor_id, started_at, resolved_at, status) VALUES
  ('inc-db-1', 'db', datetime('now', '-88 days', '+6 hours'),  datetime('now', '-88 days', '+7 hours'),  'resolved'),
  ('inc-db-2', 'db', datetime('now', '-50 days', '+22 hours'), datetime('now', '-50 days', '+23 hours'), 'resolved'),
  ('inc-db-3', 'db', datetime('now', '-25 days', '+2 hours'),  datetime('now', '-25 days', '+4 hours'),  'resolved'),
  ('inc-db-4', 'db', datetime('now', '-3 days',  '+12 hours'), datetime('now', '-3 days',  '+12 hours', '+30 minutes'), 'resolved');

-- ============================================================
-- Statistiken zum Prüfen
-- ============================================================
SELECT '=== Seed Data Summary ===' AS info;

SELECT
  m.name,
  COUNT(*)                                                  AS total_checks,
  ROUND(100.0 * SUM(CASE WHEN c.status = 'healthy' THEN 1 ELSE 0 END) / COUNT(*), 2) AS uptime_pct,
  SUM(CASE WHEN c.status = 'dead'    THEN 1 ELSE 0 END)     AS dead,
  SUM(CASE WHEN c.status = 'sick'    THEN 1 ELSE 0 END)     AS sick,
  COUNT(DISTINCT date(c.checked_at))                        AS days_covered
FROM monitors m
JOIN checks c ON c.monitor_id = m.id
GROUP BY m.id
ORDER BY uptime_pct DESC;

SELECT 'Incidents: ' || COUNT(*) FROM incidents;
