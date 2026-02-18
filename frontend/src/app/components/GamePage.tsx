import { useEffect, useMemo, useState } from "react";
import { Clock, User, X } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { apiGet, apiPost } from "../api/client";

type GameViewDto = {
  revision: number;
  state: string;
  givens: number[];   
  current: number[];  
  mistakesLeft: number;
  remainingMs: number;
  opponentProgress?: {
    filled: number;
    mistakesLeft: number;
    remainingMs: number;
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

export function GamePage() {
  const navigate = useNavigate();

  // matchId aus URL (?matchId=...) oder localStorage
  const matchId = useMemo(() => {
    const url = new URL(window.location.href);
    return url.searchParams.get("matchId") ?? localStorage.getItem("matchId") ?? "";
  }, []);

  const [selectedCell, setSelectedCell] = useState<{ row: number; col: number } | null>(null);

  const [view, setView] = useState<GameViewDto | null>(null);
  const [grid, setGrid] = useState<number[][]>(Array.from({ length: 9 }, () => Array(9).fill(0)));
  const [initialGrid, setInitialGrid] = useState<number[][]>(Array.from({ length: 9 }, () => Array(9).fill(0)));

  const [err, setErr] = useState<string | null>(null);

  // für error anzeige
  const [maxMistakes, setMaxMistakes] = useState<number | null>(null);

  useEffect(() => {
    if (!matchId) return;

    let alive = true;
    let interval: number | undefined;

    const fetchState = async () => {
      try {
        const v = await apiGet<GameViewDto>(`/match/${matchId}/state`);
        if (!alive) return;

        setView(v);
        setGrid(toGrid9x9(v.current));
        setInitialGrid(toGrid9x9(v.givens));
        setMaxMistakes((prev) => (prev == null ? v.mistakesLeft : prev));

      if (
        v.state.startsWith("Won") ||
        v.state.startsWith("Lost")
      ) {
        alive = false;
        if (interval !== undefined) clearInterval(interval);
      }
      } catch {
      }
    };

    fetchState();

    interval = window.setInterval(fetchState, 800);

    return () => {
      alive = false;
      if (interval !== undefined) clearInterval(interval);
    };
  }, [matchId]);

  const remainingSeconds = view ? Math.max(0, Math.floor(view.remainingMs / 1000)) : 0;

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
        expected_revision: view.revision,
        move_id: null,
        mv: { type: "place", cell, value: num },
      };

      const resp = await apiPost<ApplyMoveResponse>(`/match/${matchId}/move`, body);

      if (resp.view) {
        setView(resp.view);
        setGrid(toGrid9x9(resp.view.current));
        setInitialGrid(toGrid9x9(resp.view.givens));
      }

      if (resp.outcome.type === "rejected") {
        const fresh = await apiGet<GameViewDto>(`/match/${matchId}/state`);
        setView(fresh);
        setGrid(toGrid9x9(fresh.current));
        setInitialGrid(toGrid9x9(fresh.givens));
      }

      if (resp.outcome.type === "penalty" && !resp.view) {
        const fresh = await apiGet<GameViewDto>(`/match/${matchId}/state`);
        setView(fresh);
        setGrid(toGrid9x9(fresh.current));
        setInitialGrid(toGrid9x9(fresh.givens));
      }

      if (resp.outcome?.type === "won") navigate("/result/win");
      if (resp.outcome?.type === "lost") navigate("/result/lose");
    } catch (e: any) {
      setErr(e?.message ?? "Move fehlgeschlagen");
    }
  };

  const handleClear = async () => {
    if (!selectedCell || !view) return;

    const cell = cellIndex(selectedCell.row, selectedCell.col);

    try {
      setErr(null);

      const body = {
        expected_revision: view.revision,
        move_id: null,
        mv: { type: "clear", cell },
      };

      const resp = await apiPost<ApplyMoveResponse>(`/match/${matchId}/move`, body);

      if (resp.view) {
        setView(resp.view);
        setGrid(toGrid9x9(resp.view.current));
        setInitialGrid(toGrid9x9(resp.view.givens));
      }

      if (resp.outcome.type === "rejected") {
        const fresh = await apiGet<GameViewDto>(`/match/${matchId}/state`);
        setView(fresh);
        setGrid(toGrid9x9(fresh.current));
        setInitialGrid(toGrid9x9(fresh.givens));
      }
    } catch (e: any) {
      setErr(e?.message ?? "Clear fehlgeschlagen");
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
      {/* Header with Players */}
      <div className="max-w-4xl w-full mx-auto mb-3">
        <div className="grid grid-cols-2 gap-3">
          {/* Player 1 */}
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

          {/* Player 2 */}
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

      {/* Timer */}
      <div className="max-w-4xl w-full mx-auto mb-3">
        <div className="bg-white/10 backdrop-blur-lg rounded-xl p-2 border border-white/20 flex items-center justify-center gap-2">
          <Clock className="w-5 h-5 text-cyan-400" />
          <span className="text-2xl font-bold text-white tabular-nums">{formatTime(remainingSeconds)}</span>
        </div>
      </div>

      {/* Sudoku Grid */}
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
                        ${isSelected ? 'bg-cyan-400 text-slate-900' : ''}
                        ${!isSelected && isHighlighted ? 'bg-white/20' : ''}
                        ${!isSelected && !isHighlighted ? 'bg-white/5 hover:bg-white/10' : ''}
                        ${isInitial ? 'text-white' : 'text-cyan-300'}
                        ${isRightBorder ? 'border-r-2 border-white/40' : ''}
                        ${isBottomBorder ? 'border-b-2 border-white/40' : ''}
                        ${!isInitial && !isSelected ? 'cursor-pointer' : ''}
                        ${isInitial ? 'cursor-default' : ''}
                      `}
                      disabled={isInitial}
                    >
                      {cell !== 0 ? cell : ''}
                    </button>
                  );
                })
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Number Input */}
      <div className="max-w-4xl w-full mx-auto">
        <div className="bg-white/10 backdrop-blur-lg rounded-xl p-3 border border-white/20">
          <div className="grid grid-cols-10 gap-2 max-w-lg mx-auto">
            {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((num) => (
              <button
                key={num}
                onClick={() => handleNumberInput(num)}
                disabled={!selectedCell}
                className={`
                  aspect-square flex items-center justify-center text-xl font-bold rounded-lg
                  transition-all
                  ${selectedCell 
                    ? 'bg-cyan-500 hover:bg-cyan-600 text-white cursor-pointer' 
                    : 'bg-white/5 text-white/30 cursor-not-allowed'
                  }
                `}
              >
                {num}
              </button>
            ))}
            {/* Clear Button */}
            <button
              onClick={handleClear}
              disabled={!selectedCell}
              className={`
                aspect-square flex items-center justify-center text-xl font-bold rounded-lg
                transition-all
                ${
                  selectedCell
                    ? "bg-red-500 hover:bg-red-600 text-white cursor-pointer"
                    : "bg-white/5 text-white/30 cursor-not-allowed"
                }
              `}
            >
              ⌫
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

