Sudoku Royale ist ein Realtime Multiplayer Sudoku Spiel in dem zwei Spieler gegeneinander antreten.

## Features
- Sudoku Solver und Puzzle Generator
- Fehlerlimit
- Win/Lose Logik
- Live Gegner Fortschritt Tracking

## Tech Stack

### Backend
Rust
Rocket
SQLx
Tokio
Serde
PostgreSQL

### Frontend
React
Vite

### Infrastruktur
Docker 

## Setup

### Requirements
- Docker
- Rust
- Node.js

### Starten des Backends und der Datenbank
- Backend Folder in Dev Containers: Reopen in Container öffnen
- psql "$DATABASE_URL" -f migrations/001_create_users.sql im Terminal für Migration der Datenbank

### Start backend im DevContainer
- cargo run

### Start frontend
- cd frontend
- npm install
- npm run dev
- ersten Network Link abrufen (wenn beide Spieler auf einem Gerät, muss der Link einmal im Inkognito Fenster abgerufen werden)

## Sudoku Registrierungs Requirements
- Email: r"^[^@\s]+@[^@\s]+\.[^@\s]+$", zB. Test@sudoku.de
- Nutzername Requirements:
  - Zwischen 3 und 20 Zeichen lang sein
  - Nur Buchstaben, Zahlen und Unterstriche enthalten
- Passwort Requirements:
  - Mindestens 8 Zeichen lang sein
  - Mindestens einen Großbuchstaben enthalten
  - Mindestens eine Zahl enthalten
  - Mindestens ein Sonderzeichen enthalten
  z.B. Test1234!

## How to Play 
1. Registrierung und Anmeldung mit zwei verschiedenen Usern (normal + inkognito Fenster)
2. Match erstellen von einem Client und Match Link kopieren
3. Match Link bei dem zweiten Client unter Match beitreten einfügen
4. Match startet automatisch sobald beide Spieler beigetreten sind
5. Fehleranzeige für beide Spieler oben sichtbar
6. Nach Beenden eines Spiels zur Lobby zurückkehren und ein weiteres Spiel starten

## Alternatives Setup mithilfe PostgreSQL
- PostgreSQL lokal installieren, Version 17
- Datenbank erstellen
- DATABASE_URL setzen postgres://username:password@localhost:5432/sudokuroyale
- Migration ausführen mit: $env:DATABASE_URL="postgres://postgres:postgres@localhost:5432/sudokuroyale" in powershell
- cd backend
- Backend mit cargo run starten
