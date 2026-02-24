import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Clock, User, X } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { apiGet, apiPost } from "../api/client";

type GameViewDto = {
  revision: number;
  state: string;
  givens: number[];
  current: number[];
  mistakesLeft: number;
  elapsedMs: number;
  opponentProgress?: {
    filled: number;
    mistakesLeft: number;
    elapsedMs: number;
  } | null;
};

type ApplyMoveResponse = {
  outcome:
    | { type: "applied"; revision: number; applied: "placed" | "cleared" }
    | { type: "rejected"; reason: any; revision: number }
    | { type: "penalty"; reason: any; mistakesLeft: number; revision: number }
    | { type: "won"; revision: number }
    | { type: "lost"; revision: number; reason: any };
  view?: GameViewDto | null;
  replay: boolean;
};

const toGrid9x9 = (arr81: number[]) =>
  Array.from({ length: 9 }, (_, r) => arr81.slice(r * 9, r * 9 + 9));

const cellIndex = (row: number, col: number) => row * 9 + col;

const formatTime = (seconds: number) => {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
};

const secondsFromRemainingMs = (ms: number) => Math.max(0, Math.floor(ms / 1000));

function buildResultState(v: GameViewDto, maxMistakes: number | null) {
  const mm = maxMistakes ?? v.mistakesLeft;

  const myErrors = Math.max(0, mm - v.mistakesLeft);
  const oppErrors = v.opponentProgress
    ? Math.max(0, mm - v.opponentProgress.mistakesLeft)
    : 0;

  const myTime = formatTime(secondsFromRemainingMs(v.elapsedMs));
  const oppTime = v.opponentProgress
    ? formatTime(secondsFromRemainingMs(v.opponentProgress.elapsedMs))
    : "00:00";

  return {
    winState: {
      winner: { name: "Du", time: myTime, errors: myErrors },
      loser: { name: "Gegner", time: oppTime, errors: oppErrors },
      isWinner: true,
    },
    loseState: {
      winner: { name: "Gegner", time: oppTime, errors: oppErrors },
      loser: { name: "Du", time: myTime, errors: myErrors },
      isWinner: false,
    },
  };
}

type WsMsg =
  | { type: "Snapshot"; view: GameViewDto }
  | { type: "RevisionChanged"; revision: number; view: GameViewDto }
  | any;

function computeWsUrl(matchId: string) {
  const base =
    (import.meta as any).env?.VITE_API_BASE_URL ?? "http://localhost:8000";

  const u = new URL(base);
  u.protocol = u.protocol === "https:" ? "wss:" : "ws:";
  u.host = u.host.replace("127.0.0.1", "localhost");
  u.pathname = `/match/${matchId}/ws`;
  return u.toString();
}

export function GamePage() {
  const navigate = useNavigate();

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<number | null>(null);
  const backoffRef = useRef<number>(400);
  const aliveRef = useRef<boolean>(true);

  const targetEndMsRef = useRef<number | null>(null);
  const [uiRemainingSeconds, setUiRemainingSeconds] = useState<number>(0);

  const matchId = useMemo(() => {
    const url = new URL(window.location.href);
    return url.searchParams.get("matchId") ?? localStorage.getItem("matchId") ?? "";
  }, []);

  const [selectedCell, setSelectedCell] = useState<{ row: number; col: number } | null>(null);

  const [view, setView] = useState<GameViewDto | null>(null);
  const [grid, setGrid] = useState<number[][]>(Array.from({ length: 9 }, () => Array(9).fill(0)));
  const [initialGrid, setInitialGrid] = useState<number[][]>(Array.from({ length: 9 }, () => Array(9).fill(0)));

  const [err, setErr] = useState<string | null>(null);

  const navigatedRef = useRef(false);

  const [maxMistakes, setMaxMistakes] = useState<number | null>(null);
  const maxMistakesRef = useRef<number | null>(null);

  const applyView = useCallback(
    (v: GameViewDto) => {
      setView(v);
      setGrid(toGrid9x9(v.current));
      setInitialGrid(toGrid9x9(v.givens));

      if (maxMistakesRef.current == null) {
        maxMistakesRef.current = v.mistakesLeft;
      }
      const mmLocal = maxMistakesRef.current ?? v.mistakesLeft;
      setMaxMistakes((prev) => (prev == null ? mmLocal : prev));

    if (v.state?.startsWith("InProgress")) {
  
  
if (v.state === "InProgress" || v.state?.startsWith("InProgress")) {
  
  targetEndMsRef.current = Date.now() - (v.elapsedMs ?? 0);
} else {
  
  targetEndMsRef.current = null;

  
  setUiRemainingSeconds(Math.floor(((v.elapsedMs ?? 0) as number) / 1000));
}


} else {

  targetEndMsRef.current = null;
  setUiRemainingSeconds(secondsFromRemainingMs(v.elapsedMs));
}

      if (navigatedRef.current) return;

      const { winState, loseState } = buildResultState(v, mmLocal);

      if (v.state?.startsWith("Won")) {
        navigatedRef.current = true;
        navigate("/result/win", { state: winState });
        return;
      }
      if (v.state?.startsWith("Lost")) {
        navigatedRef.current = true;
        navigate("/result/lose", { state: loseState });
        return;
      }
    },
    [navigate]
  );

  useEffect(() => {
  const tick = () => {
    const startedAt = targetEndMsRef.current;

    if (startedAt == null) {
      return;
    }

    
    const elapsedMs = Math.max(0, Date.now() - startedAt);
    setUiRemainingSeconds(Math.floor(elapsedMs / 1000));
  };

  tick(); 
  const id = window.setInterval(tick, 200);
  return () => window.clearInterval(id);
}, []);

  useEffect(() => {
    if (!matchId) return;

    navigatedRef.current = false;
    maxMistakesRef.current = null;
    setMaxMistakes(null);

    aliveRef.current = true;

    const connect = () => {
      if (wsRef.current) {
        try { wsRef.current.close(); } catch {}
        wsRef.current = null;
      }

      const ws = new WebSocket(computeWsUrl(matchId));
      wsRef.current = ws;

      ws.onopen = () => {
        backoffRef.current = 400;

        apiGet<GameViewDto>(`/match/${matchId}/state`)
          .then(applyView)
          .catch(() => {});
      };

      ws.onmessage = () => {
        // Wichtig: WS-Events können nicht player-spezifische Views enthalten.
        // Daher immer die player-spezifische View per /state holen.
        apiGet<GameViewDto>(`/match/${matchId}/state`)
          .then(applyView)
          .catch(() => {});
      };

      ws.onclose = () => {
        if (!aliveRef.current) return;

        const backoff = Math.min(4000, backoffRef.current);
        backoffRef.current = Math.min(8000, backoffRef.current * 1.6);

        reconnectTimerRef.current = window.setTimeout(() => {
          connect();
        }, backoff);
      };

      ws.onerror = () => {
        try { ws.close(); } catch {}
      };
    };

    connect();

    const syncId = window.setInterval(() => {
      apiGet<GameViewDto>(`/match/${matchId}/state`)
        .then(applyView)
        .catch(() => {});
    }, 8000);

    const onVis = () => {
      if (document.visibilityState === "visible") {
        apiGet<GameViewDto>(`/match/${matchId}/state`)
          .then(applyView)
          .catch(() => {});
      }
    };
    document.addEventListener("visibilitychange", onVis);

    return () => {
      aliveRef.current = false;

      document.removeEventListener("visibilitychange", onVis);
      window.clearInterval(syncId);

      if (reconnectTimerRef.current != null) {
        window.clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }

      if (wsRef.current) {
        try { wsRef.current.close(); } catch {}
        wsRef.current = null;
      }
    };
  }, [matchId, applyView]);

  const myErrors =
    maxMistakes == null || !view ? 0 : Math.max(0, maxMistakes - view.mistakesLeft);

  const oppErrors =
    maxMistakes == null || !view?.opponentProgress
      ? 0
      : Math.max(0, maxMistakes - view.opponentProgress.mistakesLeft);

  const isInitialCell = (row: number, col: number) => initialGrid[row][col] !== 0;

  const handleCellClick = (row: number, col: number) => {
    if (initialGrid[row][col] === 0) {
      setSelectedCell({ row, col });
    }
  };

  const handleNumberInput = async (num: number) => {
    if (!selectedCell || !view) return;

    const cell = cellIndex(selectedCell.row, selectedCell.col);

    try {
      setErr(null);

      const body = {
        expectedRevision: view.revision,
        moveId: null,
        mv: { type: "place", cell, value: num },
      };

      const resp = await apiPost<ApplyMoveResponse>(`/match/${matchId}/move`, body);

      if (resp.view) applyView(resp.view);

      if (resp.outcome.type === "rejected") {
        const fresh = await apiGet<GameViewDto>(`/match/${matchId}/state`);
        applyView(fresh);
      }

      if (resp.outcome.type === "penalty" && !resp.view) {
        const fresh = await apiGet<GameViewDto>(`/match/${matchId}/state`);
        applyView(fresh);
      }
    } catch (e: any) {
      setErr(e?.message ?? "Move fehlgeschlagen");
    }
  };

  const isSameRow = (row: number) => selectedCell?.row === row;
  const isSameCol = (col: number) => selectedCell?.col === col;
  const isSameBox = (row: number, col: number) => {
    if (!selectedCell) return false;
    const boxRow = Math.floor(selectedCell.row / 3);
    const boxCol = Math.floor(selectedCell.col / 3);
    return Math.floor(row / 3) === boxRow && Math.floor(col / 3) === boxCol;
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-cyan-900 flex flex-col p-3 py-4">
      {err && (
        <div className="max-w-4xl w-full mx-auto mb-3">
          <div className="bg-red-500/20 border border-red-400/40 text-red-100 rounded-xl p-3 text-sm">
            {err}
          </div>
        </div>
      )}
      <div className="max-w-4xl w-full mx-auto mb-3">
        <div className="grid grid-cols-2 gap-3">
          <div className="bg-white/10 backdrop-blur-lg rounded-xl p-3 border border-cyan-400/50">
            <div className="flex items-center gap-2">
              <div className="bg-cyan-500 rounded-full p-2">
                <User className="w-4 h-4 text-white" />
              </div>
              <div className="flex-1">
                <div className="text-white font-bold text-sm">Du</div>
                <div className="text-red-400 text-xs flex items-center gap-1">
                  <X className="w-3 h-3" />
                  {myErrors} Fehler
                </div>
              </div>
            </div>
          </div>

          <div className="bg-white/10 backdrop-blur-lg rounded-xl p-3 border border-white/20">
            <div className="flex items-center gap-2">
              <div className="bg-white/20 rounded-full p-2">
                <User className="w-4 h-4 text-white" />
              </div>
              <div className="flex-1">
                <div className="text-white font-bold text-sm">Gegner</div>
                <div className="text-red-400 text-xs flex items-center gap-1">
                  <X className="w-3 h-3" />
                  {oppErrors} Fehler
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-4xl w-full mx-auto mb-3">
        <div className="bg-white/10 backdrop-blur-lg rounded-xl p-2 border border-white/20 flex items-center justify-center gap-2">
          <Clock className="w-5 h-5 text-cyan-400" />
          <span className="text-2xl font-bold text-white tabular-nums">{formatTime(uiRemainingSeconds)}</span>
        </div>
      </div>

      <div className="max-w-4xl w-full mx-auto mb-3">
        <div className="bg-white/10 backdrop-blur-lg rounded-xl p-3 border border-white/20">
          <div className="aspect-square max-w-lg mx-auto">
            <div className="grid grid-cols-9 gap-0 bg-white/20 p-1 rounded-lg">
              {grid.map((row, rowIndex) =>
                row.map((cell, colIndex) => {
                  const isSelected = selectedCell?.row === rowIndex && selectedCell?.col === colIndex;
                  const isHighlighted = isSameRow(rowIndex) || isSameCol(colIndex) || isSameBox(rowIndex, colIndex);
                  const isInitial = isInitialCell(rowIndex, colIndex);
                  const isRightBorder = (colIndex + 1) % 3 === 0 && colIndex !== 8;
                  const isBottomBorder = (rowIndex + 1) % 3 === 0 && rowIndex !== 8;

                  return (
                    <button
                      key={`${rowIndex}-${colIndex}`}
                      onClick={() => handleCellClick(rowIndex, colIndex)}
                      className={`
                        aspect-square flex items-center justify-center text-lg font-bold
                        transition-all
                        ${isSelected ? "bg-cyan-400 text-slate-900" : ""}
                        ${!isSelected && isHighlighted ? "bg-white/20" : ""}
                        ${!isSelected && !isHighlighted ? "bg-white/5 hover:bg-white/10" : ""}
                        ${isInitial ? "text-white" : "text-cyan-300"}
                        ${isRightBorder ? "border-r-2 border-white/40" : ""}
                        ${isBottomBorder ? "border-b-2 border-white/40" : ""}
                        ${!isInitial && !isSelected ? "cursor-pointer" : ""}
                        ${isInitial ? "cursor-default" : ""}
                      `}
                      disabled={isInitial}
                    >
                      {cell !== 0 ? cell : ""}
                    </button>
                  );
                })
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-4xl w-full mx-auto">
        <div className="bg-white/10 backdrop-blur-lg rounded-xl p-3 border border-white/20">
          <div className="grid grid-cols-9 gap-2 max-w-lg mx-auto">
            {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((num) => (
              <button
                key={num}
                onClick={() => handleNumberInput(num)}
                disabled={!selectedCell}
                className={`
                  aspect-square flex items-center justify-center text-xl font-bold rounded-lg
                  transition-all
                  ${selectedCell
                    ? "bg-cyan-500 hover:bg-cyan-600 text-white cursor-pointer"
                    : "bg-white/5 text-white/30 cursor-not-allowed"
                  }
                `}
              >
                {num}
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
