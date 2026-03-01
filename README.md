Sudoku Royale ist ein Realtime Multiplayer Sudoku Spiel in dem zwei Spieler gegeneinander antreten.

## Features
- Sudoku Solver und Puzzle Generator
- Fehlerlimit
- Win/Lose Logik
- Fehleranzeige
- Benutzerregistrierung und Login
- Match Board
- Resultpages
- Sitzungsverwaltung (Session-Management)
- Geschützte Endpunkte für authentifizierte Nutzer
- Erstellung und Beitritt zu Multiplayer-Matches
- Persistente Speicherung der Daten mit PostgreSQL

## Tech Stack

### Backend
- Rust
- Rocket
- SQLx
- Tokio
- Serde
- PostgreSQL

### Frontend
- React
- Vite

### Infrastruktur
- Docker 

## Setup für Windows, Mac möglicherweise andere Commands/Setup

### Requirements
- Docker
- Rust
- Node.js
- ggf. PostgreSQL wenn Start mit Docker nicht funktioniert

### Starten des Backends und der Datenbank
- NUR Backend Folder in einem VSCode Window öffnen
- Devcontainer Extension von Windows installieren
- Docker/Docker Desktop installieren und öffnen
- Backend Folder in Dev Containers: Reopen in Container öffnen in Strg + Shift + P
- "psql "$DATABASE_URL" -f migrations/001_create_users.sql" im Terminal für Migration der Datenbank eingeben

### Start backend im DevContainer
- cargo run

### Start frontend
- gesamtes Projekt in weiterem VSCode Window öffnen
- cd frontend
- npm install (falls Node.js nicht installiert: winget install, OpenJS.NodeJS.LTS)
- npm run dev
- ersten Network Link abrufen (wenn beide Spieler auf einem Gerät, muss der Link einmal im Inkognito Fenster abgerufen werden)

## Sudoku Registrierungs Requirements
- Email: r"^[^@\s]+@[^@\s]+\.[^@\s]+$", zB. Test@sudoku.de
- Nutzername Requirements:
  - Mindestens 1 Zeichen lang sein
- Passwort Requirements:
  - Mindestens 3 Zeichen lang sein
  z.B. Test1234!

## How to Play 
1. Registrierung und Anmeldung mit zwei verschiedenen Usern (normal + mindestens ein User im inkognito Fenster öffnen)
2. Match erstellen von einem Client und Match Link kopieren
3. Match Link bei dem zweiten Client unter Match beitreten einfügen
4. Match startet automatisch sobald beide Spieler beigetreten sind
5. Fehleranzeige für beide Spieler oben sichtbar
6. Nach Beenden eines Spiels zur Lobby zurückkehren und ein weiteres Spiel starten

## Alternatives Setup mithilfe PostgreSQL
- PostgreSQL lokal installieren, Version 17.9
- alle Components installieren, Port auf Default lassen
- Stackbuilder: PostgreSQL 17 (x64) on port 5432
- Datenbank erstellen
- CREATE DATABASE sudokuroyale
- DATABASE_URL setzen postgres://username:password@localhost:5432/sudokuroyale
- Migration ausführen mit: $env:DATABASE_URL="postgres://postgres:postgres@localhost:5432/sudokuroyale" in powershell
- cd backend
- Backend mit cargo run starten

## extern Tools Usage
- ChatGPT
  - WS und DTO's größtenteils unter Zuhilfenahme von KI erstellt für Match
  - Unterstützen für Implementierung von Serializing
  - Revision Handling
  - Allgemeine Unterstützung beim Error-Handling
  - Unterstützung bei der Einrichtung von Docker und DevContainer
  - Unterstützung beim Fehlerkorrigeren während der Umstellung von In-Memory-Logik auf     eine Datenbankanbindung
  - Unterstützung beim Debugging einzelner Backend-Routen
  - Formulierung einzelner API-Wrapper im Frontend
  - Unterstützung bei der Strukturierung einzelner Servicefunktionen im Game-Match Modul

- Figma
  - UI Design wurde mit Hilfe von Figma erstellt und anschließend entsprechend ergänzt/angepasst

## Testing
- Tests zum Überprüfen von Generator, ob Sudokus korrekt generiert werden
- Tests zum Überprüfen der Solve Algorithmen
- Tests der Board Repräsentation und Board Erstellung

