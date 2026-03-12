import { useState, useCallback, useRef } from "react";

interface WordEntry {
  rank: number;
  word: string;
  count: number;
  percent: number;
}

interface AnalyzeResponse {
  words: WordEntry[];
  total_words: number;
  total_unique: number;
  scoped_messages: number;
}

type Speaker = "user" | "assistant" | "both";
type SortKey = "rank" | "word" | "count" | "percent";
type SortDir = "asc" | "desc";

export default function App() {
  const [results, setResults] = useState<AnalyzeResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [rawJson, setRawJson] = useState<string | null>(null);

  // Controls
  const [normalize, setNormalize] = useState(true);
  const [stopWords, setStopWords] = useState(false);
  const [speaker, setSpeaker] = useState<Speaker>("user");
  const [limit, setLimit] = useState(100);

  // Sort
  const [sortKey, setSortKey] = useState<SortKey>("rank");
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  // View
  const [view, setView] = useState<"table" | "chart">("table");

  const fileRef = useRef<HTMLInputElement>(null);
  const dropRef = useRef<HTMLDivElement>(null);

  const analyze = useCallback(
    async (data: string) => {
      setLoading(true);
      setError(null);
      try {
        const res = await fetch("/api/analyze", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            data,
            normalize,
            stop_words: stopWords,
            speaker,
            limit,
          }),
        });
        if (!res.ok) {
          const text = await res.text();
          throw new Error(text || res.statusText);
        }
        const json: AnalyzeResponse = await res.json();
        setResults(json);
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : "Unknown error");
      } finally {
        setLoading(false);
      }
    },
    [normalize, stopWords, speaker, limit]
  );

  const handleFile = useCallback(
    (file: File) => {
      setFileName(file.name);
      const reader = new FileReader();
      reader.onload = (e) => {
        const text = e.target?.result as string;
        setRawJson(text);
        analyze(text);
      };
      reader.readAsText(file);
    },
    [analyze]
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      dropRef.current?.classList.remove("drag-over");
      const file = e.dataTransfer.files[0];
      if (file) handleFile(file);
    },
    [handleFile]
  );

  const reanalyze = useCallback(() => {
    if (rawJson) analyze(rawJson);
  }, [rawJson, analyze]);

  const sorted = results
    ? [...results.words].sort((a, b) => {
        let cmp = 0;
        if (sortKey === "word") cmp = a.word.localeCompare(b.word);
        else cmp = (a[sortKey] as number) - (b[sortKey] as number);
        return sortDir === "desc" ? -cmp : cmp;
      })
    : [];

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir(key === "word" ? "asc" : "desc");
    }
  };

  const maxCount = sorted.length > 0 ? sorted[0].count : 1;
  const topForChart = sorted.slice(0, 40);

  return (
    <div className="app">
      <header className="header">
        <h1 className="logo">
          <span className="logo-icon">λ</span> lexis
        </h1>
        <span className="tagline">Word frequency analyzer for Claude exports</span>
      </header>

      {/* Upload Zone */}
      <div
        ref={dropRef}
        className="dropzone"
        onDragOver={(e) => {
          e.preventDefault();
          dropRef.current?.classList.add("drag-over");
        }}
        onDragLeave={() => dropRef.current?.classList.remove("drag-over")}
        onDrop={handleDrop}
        onClick={() => fileRef.current?.click()}
      >
        <input
          ref={fileRef}
          type="file"
          accept=".json"
          style={{ display: "none" }}
          onChange={(e) => {
            const f = e.target.files?.[0];
            if (f) handleFile(f);
          }}
        />
        {fileName ? (
          <div className="dropzone-loaded">
            <span className="file-icon">📄</span>
            <span className="file-name">{fileName}</span>
            <span className="file-hint">click or drop to replace</span>
          </div>
        ) : (
          <div className="dropzone-empty">
            <span className="upload-icon">↑</span>
            <span>Drop a Claude JSON export here, or click to browse</span>
          </div>
        )}
      </div>

      {/* Controls */}
      <div className="controls">
        <div className="control-group">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={normalize}
              onChange={(e) => setNormalize(e.target.checked)}
            />
            <span className="toggle-text">Normalize</span>
          </label>
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={stopWords}
              onChange={(e) => setStopWords(e.target.checked)}
            />
            <span className="toggle-text">Filter stop words</span>
          </label>
        </div>

        <div className="control-group">
          <label className="select-label">
            Speaker
            <select value={speaker} onChange={(e) => setSpeaker(e.target.value as Speaker)}>
              <option value="user">User</option>
              <option value="assistant">Assistant</option>
              <option value="both">Both</option>
            </select>
          </label>
          <label className="select-label">
            Limit
            <input
              type="number"
              min={0}
              value={limit}
              onChange={(e) => setLimit(parseInt(e.target.value) || 0)}
              className="limit-input"
            />
          </label>
        </div>

        <button className="btn-analyze" onClick={reanalyze} disabled={!rawJson || loading}>
          {loading ? "Analyzing..." : "Re-analyze"}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {/* Stats Bar */}
      {results && (
        <div className="stats">
          <div className="stat">
            <span className="stat-value">{results.total_words.toLocaleString()}</span>
            <span className="stat-label">total words</span>
          </div>
          <div className="stat">
            <span className="stat-value">{results.total_unique.toLocaleString()}</span>
            <span className="stat-label">unique words</span>
          </div>
          <div className="stat">
            <span className="stat-value">{results.scoped_messages.toLocaleString()}</span>
            <span className="stat-label">messages scoped</span>
          </div>
          <div className="stat">
            <span className="stat-value">{results.words.length.toLocaleString()}</span>
            <span className="stat-label">rows shown</span>
          </div>
        </div>
      )}

      {/* View Toggle */}
      {results && (
        <div className="view-toggle">
          <button className={view === "table" ? "active" : ""} onClick={() => setView("table")}>
            Table
          </button>
          <button className={view === "chart" ? "active" : ""} onClick={() => setView("chart")}>
            Chart
          </button>
        </div>
      )}

      {/* Table View */}
      {results && view === "table" && (
        <div className="table-wrap">
          <table className="freq-table">
            <thead>
              <tr>
                {(["rank", "word", "count", "percent"] as SortKey[]).map((key) => (
                  <th key={key} onClick={() => handleSort(key)} className="sortable">
                    {key === "percent" ? "%" : key.charAt(0).toUpperCase() + key.slice(1)}
                    {sortKey === key && (
                      <span className="sort-arrow">{sortDir === "asc" ? " ▲" : " ▼"}</span>
                    )}
                  </th>
                ))}
                <th className="bar-header">Distribution</th>
              </tr>
            </thead>
            <tbody>
              {sorted.map((w) => (
                <tr key={w.rank + w.word}>
                  <td className="cell-rank">{w.rank}</td>
                  <td className="cell-word">{w.word}</td>
                  <td className="cell-count">{w.count.toLocaleString()}</td>
                  <td className="cell-pct">{w.percent.toFixed(2)}%</td>
                  <td className="cell-bar">
                    <div className="bar" style={{ width: `${(w.count / maxCount) * 100}%` }} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Chart View */}
      {results && view === "chart" && (
        <div className="chart">
          {topForChart.map((w) => (
            <div key={w.word} className="chart-row">
              <span className="chart-label">{w.word}</span>
              <div className="chart-bar-wrap">
                <div
                  className="chart-bar"
                  style={{ width: `${(w.count / maxCount) * 100}%` }}
                />
              </div>
              <span className="chart-value">{w.count.toLocaleString()}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
