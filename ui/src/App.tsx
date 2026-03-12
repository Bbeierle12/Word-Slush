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

type SortKey = "rank" | "word" | "count" | "percent";
type SortDir = "asc" | "desc";

function ResultsSection({
  title,
  results,
}: {
  title: string;
  results: AnalyzeResponse;
}) {
  const [sortKey, setSortKey] = useState<SortKey>("rank");
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [view, setView] = useState<"table" | "chart">("table");

  const sorted = [...results.words].sort((a, b) => {
    let cmp = 0;
    if (sortKey === "word") cmp = a.word.localeCompare(b.word);
    else cmp = (a[sortKey] as number) - (b[sortKey] as number);
    return sortDir === "desc" ? -cmp : cmp;
  });

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir(key === "word" ? "asc" : "desc");
    }
  };

  const maxCount = sorted.length > 0 ? Math.max(...sorted.map((w) => w.count)) : 1;
  const topForChart = sorted.slice(0, 40);

  return (
    <div className="results-section">
      <h2 className="section-title">{title}</h2>

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
          <span className="stat-label">messages</span>
        </div>
        <div className="stat">
          <span className="stat-value">{results.words.length.toLocaleString()}</span>
          <span className="stat-label">rows shown</span>
        </div>
      </div>

      <div className="view-toggle">
        <button className={view === "table" ? "active" : ""} onClick={() => setView("table")}>
          Table
        </button>
        <button className={view === "chart" ? "active" : ""} onClick={() => setView("chart")}>
          Chart
        </button>
      </div>

      {view === "table" && (
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

      {view === "chart" && (
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

export default function App() {
  const [userResults, setUserResults] = useState<AnalyzeResponse | null>(null);
  const [assistantResults, setAssistantResults] = useState<AnalyzeResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [rawFile, setRawFile] = useState<File | null>(null);

  const [normalize, setNormalize] = useState(true);
  const [stopWords, setStopWords] = useState(false);
  const [limit, setLimit] = useState(100);

  const fileRef = useRef<HTMLInputElement>(null);
  const dropRef = useRef<HTMLDivElement>(null);

  const analyzeForSpeaker = useCallback(
    async (file: File, speaker: string): Promise<AnalyzeResponse> => {
      const form = new FormData();
      form.append("file", file);
      form.append("normalize", String(normalize));
      form.append("stop_words", String(stopWords));
      form.append("speaker", speaker);
      form.append("limit", String(limit));

      const res = await fetch("/api/analyze", { method: "POST", body: form });
      if (!res.ok) {
        const text = await res.text();
        throw new Error(text || res.statusText);
      }
      return res.json();
    },
    [normalize, stopWords, limit]
  );

  const analyze = useCallback(
    async (file: File) => {
      setLoading(true);
      setError(null);
      try {
        const [user, assistant] = await Promise.all([
          analyzeForSpeaker(file, "user"),
          analyzeForSpeaker(file, "assistant"),
        ]);
        setUserResults(user);
        setAssistantResults(assistant);
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : "Unknown error");
      } finally {
        setLoading(false);
      }
    },
    [analyzeForSpeaker]
  );

  const handleFile = useCallback(
    (file: File) => {
      setFileName(file.name);
      setRawFile(file);
      analyze(file);
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
    if (rawFile) analyze(rawFile);
  }, [rawFile, analyze]);

  return (
    <div className="app">
      <header className="header">
        <h1 className="logo">
          <span className="logo-icon">&lambda;</span> lexis
        </h1>
        <span className="tagline">Word frequency analyzer</span>
      </header>

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
          accept=".json,.zip,.txt,.md,.log,.csv,.html,.xml,.yaml,.yml,.toml,.rs,.py,.js,.ts,.go,.java,.c,.cpp,.rb,.sh"
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
            <span>Drop a file here (.json, .zip, .txt, .md, etc.) or click to browse</span>
          </div>
        )}
      </div>

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

        <button className="btn-analyze" onClick={reanalyze} disabled={!rawFile || loading}>
          {loading ? "Analyzing..." : "Re-analyze"}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {(userResults || assistantResults) && (
        <div className="split-view">
          {userResults && <ResultsSection title="Your Words" results={userResults} />}
          {assistantResults && (
            <ResultsSection title="Claude's Words" results={assistantResults} />
          )}
        </div>
      )}
    </div>
  );
}
