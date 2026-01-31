import { useState, useEffect } from 'react';
import { Clock, User, X } from 'lucide-react';
import { useNavigate } from "react-router-dom";


interface PlayerInfo {
  name: string;
  errors: number;
}

export function GamePage() {
  const [player1] = useState<PlayerInfo>({ name: 'Du', errors: 0 });
  const [player2] = useState<PlayerInfo>({ name: 'Gegner', errors: 1 });
  const [timer, setTimer] = useState(0);
  const [selectedCell, setSelectedCell] = useState<{ row: number; col: number } | null>(null);

  const navigate = useNavigate();

  // Mock Sudoku grid (0 means empty cell)
  const [grid, setGrid] = useState<number[][]>([
    [5, 3, 0, 0, 7, 0, 0, 0, 0],
    [6, 0, 0, 1, 9, 5, 0, 0, 0],
    [0, 9, 8, 0, 0, 0, 0, 6, 0],
    [8, 0, 0, 0, 6, 0, 0, 0, 3],
    [4, 0, 0, 8, 0, 3, 0, 0, 1],
    [7, 0, 0, 0, 2, 0, 0, 0, 6],
    [0, 6, 0, 0, 0, 0, 2, 8, 0],
    [0, 0, 0, 4, 1, 9, 0, 0, 5],
    [0, 0, 0, 0, 8, 0, 0, 7, 9],
  ]);

  // Pre-filled cells (not editable)
  const initialGrid = [
    [5, 3, 0, 0, 7, 0, 0, 0, 0],
    [6, 0, 0, 1, 9, 5, 0, 0, 0],
    [0, 9, 8, 0, 0, 0, 0, 6, 0],
    [8, 0, 0, 0, 6, 0, 0, 0, 3],
    [4, 0, 0, 8, 0, 3, 0, 0, 1],
    [7, 0, 0, 0, 2, 0, 0, 0, 6],
    [0, 6, 0, 0, 0, 0, 2, 8, 0],
    [0, 0, 0, 4, 1, 9, 0, 0, 5],
    [0, 0, 0, 0, 8, 0, 0, 7, 9],
  ];

  // Timer effect
  useEffect(() => {
    const interval = setInterval(() => {
      setTimer((prev) => prev + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const handleCellClick = (row: number, col: number) => {
    if (initialGrid[row][col] === 0) {
      setSelectedCell({ row, col });
    }
  };

  const handleNumberInput = (num: number) => {
    if (selectedCell) {
      const newGrid = [...grid];
      newGrid[selectedCell.row][selectedCell.col] = num;
      setGrid(newGrid);
    }
  };

  const isInitialCell = (row: number, col: number) => {
    return initialGrid[row][col] !== 0;
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
                <div className="text-white font-bold text-sm">{player1.name}</div>
                <div className="text-red-400 text-xs flex items-center gap-1">
                  <X className="w-3 h-3" />
                  {player1.errors} Fehler
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
                <div className="text-white font-bold text-sm">{player2.name}</div>
                <div className="text-red-400 text-xs flex items-center gap-1">
                  <X className="w-3 h-3" />
                  {player2.errors} Fehler
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
          <span className="text-2xl font-bold text-white tabular-nums">{formatTime(timer)}</span>
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
                    ? 'bg-cyan-500 hover:bg-cyan-600 text-white cursor-pointer' 
                    : 'bg-white/5 text-white/30 cursor-not-allowed'
                  }
                `}
              >
                {num}
              </button>
            ))}
          </div>
        </div>
      </div>
            {/* TEMP: Test Buttons (später löschen) */}
      <div className="max-w-4xl w-full mx-auto mt-3">
        <div className="grid grid-cols-2 gap-3">
          <button
            onClick={() => navigate("/result/win")}
            className="bg-green-500/20 hover:bg-green-500/30 border border-green-400/50 text-green-200 font-semibold py-3 rounded-xl transition-all"
          >
            Test: Win
          </button>

          <button
            onClick={() => navigate("/result/lose")}
            className="bg-red-500/20 hover:bg-red-500/30 border border-red-400/50 text-red-200 font-semibold py-3 rounded-xl transition-all"
          >
            Test: Lose
          </button>
        </div>
      </div>
    </div>
  );
}