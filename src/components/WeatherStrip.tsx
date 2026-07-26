import type { WeatherSummary } from "../types";

/**
 * One-line weather context under the tabs.
 *
 * Its job is to explain the schedule, not to be a weather app: when
 * watering has been pushed out because it rained, the widget should say
 * so rather than leaving the user wondering why nothing is due.
 */
export function WeatherStrip({ weather }: { weather: WeatherSummary | null }) {
  if (!weather) return null;

  const temp =
    weather.today_max_c !== null
      ? `${Math.round(weather.today_max_c)}°${
          weather.today_min_c !== null ? ` / ${Math.round(weather.today_min_c)}°` : ""
        }`
      : null;

  return (
    <div className="weather-strip" title={`Weather for ${weather.location_label}`}>
      <span className="weather-place ellipsis">{weather.location_label}</span>
      {temp && <span className="weather-temp">{temp}</span>}
      {weather.rained_recently ? (
        <span className="weather-rain" title={`${weather.recent_rain_mm}mm in the last few days`}>
          <RainIcon /> watering eased off
        </span>
      ) : (
        weather.recent_rain_mm > 0 && (
          <span className="weather-rain-quiet">{weather.recent_rain_mm}mm recently</span>
        )
      )}
      {weather.stale && (
        <span className="weather-stale" title="Couldn't refresh recently; showing the last data fetched">
          offline
        </span>
      )}
    </div>
  );
}

function RainIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round">
      <path d="M4 14a4 4 0 0 1 3-6.9 5 5 0 0 1 9.5 1.4A3.5 3.5 0 0 1 19 14" />
      <path d="M8 18v2M12 18v3M16 18v2" />
    </svg>
  );
}
