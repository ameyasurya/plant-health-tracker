import { useEffect, useRef, useState } from "react";
import { Field, Panel } from "./Panel";
import { api } from "../api";
import type { Location, Settings } from "../types";

interface Props {
  settings: Settings;
  onClose: () => void;
  onSave: (settings: Settings) => Promise<void>;
}

export function SettingsPanel({ settings, onClose, onSave }: Props) {
  const [notificationTime, setNotificationTime] = useState(settings.notification_time);
  const [launchAtStartup, setLaunchAtStartup] = useState(settings.launch_at_startup);
  const [weatherEnabled, setWeatherEnabled] = useState(settings.weather_enabled);
  const [location, setLocation] = useState<Location | null>(settings.location);

  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Location[]>([]);
  const [searching, setSearching] = useState(false);
  const [detecting, setDetecting] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  // Reflect what Windows actually has registered, not just our stored
  // preference -- they can disagree if the user changed it elsewhere.
  const [osAutostart, setOsAutostart] = useState<boolean | null>(null);
  useEffect(() => {
    api.isAutostartEnabled().then(setOsAutostart).catch(() => setOsAutostart(null));
  }, []);

  const timerRef = useRef<number | undefined>(undefined);
  useEffect(() => {
    window.clearTimeout(timerRef.current);
    const q = query.trim();
    if (q.length < 2) {
      setResults([]);
      return;
    }
    timerRef.current = window.setTimeout(() => {
      setSearching(true);
      setError(null);
      api
        .searchPlaces(q)
        .then(setResults)
        .catch((e) => setError(String(e)))
        .finally(() => setSearching(false));
    }, 350);
    return () => window.clearTimeout(timerRef.current);
  }, [query]);

  async function handleDetect() {
    setDetecting(true);
    setError(null);
    try {
      setLocation(await api.detectLocation());
      setQuery("");
      setResults([]);
    } catch (e) {
      setError(String(e));
    }
    setDetecting(false);
  }

  async function handleSave() {
    setBusy(true);
    setError(null);
    try {
      await onSave({
        ...settings,
        notification_time: notificationTime,
        launch_at_startup: launchAtStartup,
        weather_enabled: weatherEnabled,
        location,
      });
      setSaved(true);
      if (launchAtStartup !== osAutostart) {
        api.isAutostartEnabled().then(setOsAutostart).catch(() => {});
      }
    } catch (e) {
      setError(String(e));
    }
    setBusy(false);
  }

  return (
    <Panel title="Settings" onClose={onClose}>
      <section className="settings-group">
        <h4>Location</h4>
        <p className="panel-hint">
          Used for your local day and to adjust watering to real rainfall and heat. Everything works
          without it; the schedule just falls back to month-based seasons.
        </p>

        {location ? (
          <div className="location-current">
            <div className="location-label ellipsis" title={location.label}>
              {location.label}
            </div>
            <div className="location-meta">
              {location.latitude.toFixed(2)}, {location.longitude.toFixed(2)} · {location.timezone}
            </div>
            <button className="danger-link" onClick={() => setLocation(null)}>
              Clear location
            </button>
          </div>
        ) : (
          <div className="location-empty">No location set.</div>
        )}

        <div className="search-row">
          <input
            value={query}
            placeholder="Search a city..."
            onChange={(e) => setQuery(e.target.value)}
          />
          <button
            className="search-manual-btn"
            disabled={detecting}
            onClick={handleDetect}
            title="Look up your approximate location from your IP address. Only happens when you press this."
          >
            {detecting ? "..." : "Detect"}
          </button>
        </div>

        {searching && <div className="settings-note">Searching...</div>}
        <ul className="catalog-results">
          {results.map((r) => (
            <li key={`${r.latitude},${r.longitude}`}>
              <button
                className="catalog-hit"
                onClick={() => {
                  setLocation(r);
                  setQuery("");
                  setResults([]);
                }}
              >
                <span className="catalog-name ellipsis">{r.label}</span>
                <span className="catalog-sci ellipsis">{r.timezone}</span>
              </button>
            </li>
          ))}
        </ul>

        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={weatherEnabled}
            onChange={(e) => setWeatherEnabled(e.target.checked)}
          />
          <span>Use weather to adjust watering</span>
        </label>
        {!weatherEnabled && (
          <div className="settings-note">With this off the app makes no network requests at all.</div>
        )}
      </section>

      <section className="settings-group">
        <h4>Reminders</h4>
        <Field label="Daily digest time">
          <input
            type="time"
            value={notificationTime}
            onChange={(e) => setNotificationTime(e.target.value)}
          />
        </Field>
        <p className="panel-hint">One notification a day listing everything due, not one per plant.</p>
      </section>

      <section className="settings-group">
        <h4>Startup</h4>
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={launchAtStartup}
            onChange={(e) => setLaunchAtStartup(e.target.checked)}
          />
          <span>Start with Windows (hidden to tray)</span>
        </label>
        {osAutostart !== null && osAutostart !== settings.launch_at_startup && (
          <div className="settings-note">
            Windows currently has this {osAutostart ? "enabled" : "disabled"}. Saving will apply
            your choice.
          </div>
        )}
      </section>

      {error && <div className="panel-error">{error}</div>}
      {saved && !error && <div className="settings-saved">Saved.</div>}

      <div className="panel-actions">
        <button onClick={onClose}>Close</button>
        <button className="btn-primary" disabled={busy} onClick={handleSave}>
          {busy ? "Saving..." : "Save"}
        </button>
      </div>
    </Panel>
  );
}
