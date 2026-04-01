# E2E Test Script

## Prerequisites
1. SpacetimeDB running locally: `spacetime start`
2. Server module published: `spacetime publish collaborate crates/server`
3. App running: `cd crates/app && trunk serve`

## Test Cases

### 1. Canvas Loads
- [ ] Open http://localhost:8080
- [ ] See "NORMAL" mode indicator in top bar
- [ ] See "Offline" or "Connected" in top bar
- [ ] Canvas fills the viewport

### 2. Vim Mode - Shape Creation
- [ ] Press `r` — mode changes to "INSERT"
- [ ] Click and drag on canvas — rectangle appears
- [ ] Press `Escape` — mode returns to "NORMAL"
- [ ] Press `e` then click+drag — ellipse appears

### 3. Vim Mode - Navigation
- [ ] Select a shape (click it)
- [ ] Press `h` — shape moves left
- [ ] Press `j` — shape moves down
- [ ] Press `k` — shape moves up
- [ ] Press `l` — shape moves right

### 4. Vim Mode - Delete/Copy/Paste
- [ ] Select a shape, press `dd` — shape deleted
- [ ] Press `u` — shape restored (undo)
- [ ] Select a shape, press `yy` then `p` — shape duplicated

### 5. Vim Mode - Commands
- [ ] Press `:` — command bar activates
- [ ] Type `color #ff0000` + Enter — stroke color changes to red
- [ ] Draw a new shape — it appears red
- [ ] Press `:w` + Enter — PNG downloads

### 6. Multi-user (if SpacetimeDB connected)
- [ ] Open two browser tabs
- [ ] Draw in tab 1 — appears in tab 2
- [ ] Move mouse in tab 1 — cursor visible in tab 2

### 7. Freehand Drawing
- [ ] Press `f` — enter freehand mode
- [ ] Click and drag — freehand stroke appears
- [ ] Press `Escape` — return to normal

### 8. Text
- [ ] Press `t` — enter text mode
- [ ] Click canvas — cursor appears
- [ ] Type text — text appears on canvas
- [ ] Press `Escape` — text committed

### 9. Zoom
- [ ] Scroll wheel up — canvas zooms in
- [ ] Scroll wheel down — canvas zooms out
- [ ] Bottom bar shows zoom percentage

### 10. SVG Export
- [ ] Draw several shapes
- [ ] Press `:ws` + Enter — SVG file downloads
- [ ] Open SVG in browser — matches canvas content
