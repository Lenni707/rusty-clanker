# SMC Implementierungs-Anleitung für Hidden Gems

## Das Problem (nochmal klar formuliert)

**Gegeben:**
- Bot-Position: `[x, y]`
- **EIN** Signal-Level: `0.0` bis `3.0` (Summe aller Gems!)
- Formel für eine Gem: `signal = max(0, 1 - distance/signal_radius)`
- **Wichtig**: Bei mehreren Gems werden die Signale ADDIERT!

**Gesucht:**
- Wo liegen die 1-3 Gems?

**Herausforderung:**
- Du misst nur die Summe, nicht einzelne Signale
- Du kennst die Anzahl der Gems nicht
- Millionen mögliche Kombinationen

---

## Lösung: Sequential Monte Carlo (SMC)

### Kernidee
Wir raten viele mögliche "Welten" (Partikel) und bewerten sie:
- Jede Welt sagt: "Hier liegen X Gems an Position Y"
- Wir berechnen: "Welches Signal würde ich in dieser Welt messen?"
- Welten, die zum echten Signal passen → behalten
- Welten, die nicht passen → wegwerfen

---

## Structs die du brauchst

### 1. `GemHypothesis` - Eine vermutete Gem
```rust
pub struct GemHypothesis {
    pub position: [i32; 2],
}
```
**Was ist das?**
- Einfach nur eine Position wo eine Gem SEIN KÖNNTE
- Jeder Partikel hat 1-3 davon

---

### 2. `Particle` - Eine "Welt-Hypothese"
```rust
pub struct Particle {
    pub gems: Vec<GemHypothesis>,  // 1-3 Gems
    pub weight: f32,                // Wie gut passt diese Welt?
}
```
**Was ist das?**
- Eine komplette Hypothese: "Ich glaube es gibt 2 Gems bei [10,5] und [30,20]"
- `weight`: Wahrscheinlichkeit dass diese Welt stimmt (0.0 - 1.0)
- Alle weights zusammen = 1.0

**Wichtige Funktion:**
```rust
impl Particle {
    fn calc_expected_signal(&self, bot_pos: [i32; 2], signal_radius: f32) -> f32 {
        let mut total = 0.0;
        for gem in &self.gems {
            let dist = manhattan_distance(bot_pos, gem.position);
            let signal = calc_signal_strength(dist, signal_radius);
            total += signal;  // ADDIEREN! Das ist der Trick!
        }
        total
    }
}
```
**Was macht die Funktion?**
- Berechnet welches Signal ich bei `bot_pos` messen WÜRDE, wenn diese Welt wahr wäre
- Summiert über alle Gems in dieser Hypothese

---

### 3. `ParticleFilter` - Der eigentliche Algorithmus
```rust
pub struct ParticleFilter {
    particles: Vec<Particle>,      // Alle Hypothesen (z.B. 1000 Stück)
    num_particles: usize,          // Wie viele Partikel?
    width: i32,                     // Spielfeld-Größe
    height: i32,
    signal_radius: f32,
    rng: rand::rngs::ThreadRng,    // Zufallsgenerator
}
```

---

## Funktionen die du brauchst

### 1. `initialize()` - Erstelle random Hypothesen
```rust
pub fn initialize(&mut self) {
    self.particles.clear();
    
    for _ in 0..self.num_particles {
        // Zufällige Anzahl Gems (1-3)
        let num_gems = self.rng.gen_range(1..=3);
        
        let mut gems = Vec::new();
        for _ in 0..num_gems {
            // Zufällige Position
            let x = self.rng.gen_range(0..self.width);
            let y = self.rng.gen_range(0..self.height);
            gems.push(GemHypothesis { position: [x, y] });
        }
        
        self.particles.push(Particle {
            gems,
            weight: 1.0 / self.num_particles as f32,
        });
    }
}
```
**Was macht sie?**
- Erstellt z.B. 1000 komplett zufällige Hypothesen
- Jede Hypothese hat 1-3 Gems an zufälligen Orten
- Alle starten mit gleichem Gewicht

**Wann aufrufen?**
- Einmal am Anfang des Spiels

---

### 2. `update()` - Bewerte Partikel anhand Messung
```rust
pub fn update(&mut self, bot_pos: [i32; 2], observed_signal: f32) {
    let mut total_weight = 0.0;
    
    // Für jeden Partikel:
    for particle in &mut self.particles {
        // Was würde ich in dieser Welt messen?
        let expected = particle.calc_expected_signal(bot_pos, self.signal_radius);
        
        // Wie gut passt das zum echten Signal?
        let diff = (expected - observed_signal).abs();
        
        // Gaussian Likelihood: Je kleiner diff, desto höher likelihood
        let sigma = 0.1;  // "Messrauschen" - tuning parameter!
        let likelihood = (-diff * diff / (2.0 * sigma * sigma)).exp();
        
        particle.weight = likelihood;
        total_weight += likelihood;
    }
    
    // Normalisiere: Alle weights zusammen = 1.0
    if total_weight > 0.0 {
        for particle in &mut self.particles {
            particle.weight /= total_weight;
        }
    }
}
```
**Was macht sie?**
- Nimmt eine neue Messung: Bot ist bei `bot_pos`, misst `observed_signal`
- Für jeden Partikel: Berechne "expected signal" und vergleiche mit echter Messung
- Partikel die gut passen → hohes Gewicht
- Partikel die schlecht passen → niedriges Gewicht

**Beispiel:**
```
Messung: Bot bei [10,10], Signal = 0.7

Partikel A: 1 Gem bei [15,12]
  → expected = 0.65
  → diff = 0.05
  → weight = hoch ✓

Partikel B: 2 Gems bei [40,40] und [45,45]
  → expected = 0.01
  → diff = 0.69
  → weight = sehr niedrig ✗
```

**Wann aufrufen?**
- Nach JEDER Messung (jeder Tick wo signal_level > 0.0)

---

### 3. `resample()` - Entferne schlechte Partikel
```rust
pub fn resample(&mut self) {
    let mut new_particles = Vec::new();
    
    // Systematic Resampling (effizienter als naive Methode)
    let mut cumulative = Vec::new();
    let mut sum = 0.0;
    for p in &self.particles {
        sum += p.weight;
        cumulative.push(sum);
    }
    
    // Ziehe num_particles neue Partikel basierend auf Gewichten
    let step = 1.0 / self.num_particles as f32;
    let start: f32 = self.rng.gen_range(0.0..step);
    
    let mut idx = 0;
    for i in 0..self.num_particles {
        let target = start + i as f32 * step;
        
        while idx < cumulative.len() - 1 && cumulative[idx] < target {
            idx += 1;
        }
        
        new_particles.push(self.particles[idx].clone());
    }
    
    self.particles = new_particles;
    
    // Reset weights
    for p in &mut self.particles {
        p.weight = 1.0 / self.num_particles as f32;
    }
}
```
**Was macht sie?**
- Schlechte Partikel (niedriges Gewicht) werden gelöscht
- Gute Partikel (hohes Gewicht) werden mehrfach kopiert
- Danach haben wieder alle Partikel gleiches Gewicht

**Metapher:**
- Vor Resampling: 1000 Partikel mit unterschiedlichen Gewichten
- Nach Resampling: 1000 Partikel, aber gute wurden x-mal kopiert, schlechte sind weg

**Wann aufrufen?**
- Wenn die "Effective Sample Size" zu klein wird
- Oder: Nach jeder Messung (einfacher)

---

### 4. `jitter()` - Verbessere Partikel durch Mutation
```rust
pub fn jitter(&mut self) {
    for particle in &mut self.particles {
        let action: f32 = self.rng.gen();
        
        if action < 0.6 {
            // 60%: Verschiebe eine Gem leicht
            if !particle.gems.is_empty() { // achte darauf, das du solange korriegierst, bis alle gems auf einer validen position sind
                let idx = self.rng.gen_range(0..particle.gems.len());
                let dx = self.rng.gen_range(-2..=2);
                let dy = self.rng.gen_range(-2..=2);
                
                let new_x = (particle.gems[idx].position[0] + dx)
                    .max(0).min(self.width - 1);
                let new_y = (particle.gems[idx].position[1] + dy)
                    .max(0).min(self.height - 1);
                
                particle.gems[idx].position = [new_x, new_y];
            }
        } else if action < 0.8 && particle.gems.len() < 3 {
            // 20%: Füge Gem hinzu (wenn < 3)
            let x = self.rng.gen_range(0..self.width);
            let y = self.rng.gen_range(0..self.height);
            particle.gems.push(GemHypothesis { position: [x, y] });
        } else if particle.gems.len() > 1 {
            // 20%: Entferne Gem (wenn > 1)
            let idx = self.rng.gen_range(0..particle.gems.len());
            particle.gems.remove(idx);
        }
    }
}
```
**Was macht sie?**
- Fügt kleine Zufallsänderungen zu Partikeln hinzu
- Verhindert dass alle Partikel identisch werden
- Erlaubt "Exploration" von neuen Hypothesen

**Arten von Moves:**
1. **Verschieben**: Gem um ±2 Felder bewegen → Verfeinerung
2. **Hinzufügen**: Neue Gem spawnen → Teste "gibt es noch eine?"
3. **Entfernen**: Gem löschen → Teste "brauche ich diese?"

**Wann aufrufen?**
- Nach jedem Resample

---

### 5. `get_best_gem_estimate()` - Hole Ergebnis
```rust
pub fn get_best_gem_estimate(&self) -> Option<[i32; 2]> {
    // Grid-basiertes Clustering
    let grid_size = 3;
    let mut grid_weights: HashMap<(i32, i32), f32> = HashMap::new();
    
    // Zähle Gewichte pro Grid-Zelle
    for particle in &self.particles {
        for gem in &particle.gems {
            let grid_x = gem.position[0] / grid_size;
            let grid_y = gem.position[1] / grid_size;
            *grid_weights.entry((grid_x, grid_y)).or_insert(0.0) += particle.weight;
        }
    }
    
    // Finde beste Zelle
    let best_cell = grid_weights.iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())?;
    
    // Berechne gewichteten Durchschnitt in dieser Zelle
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut total_weight = 0.0;
    
    for particle in &self.particles {
        for gem in &particle.gems {
            let grid_x = gem.position[0] / grid_size;
            let grid_y = gem.position[1] / grid_size;
            
            if (grid_x, grid_y) == *best_cell.0 {
                sum_x += gem.position[0] as f32 * particle.weight;
                sum_y += gem.position[1] as f32 * particle.weight;
                total_weight += particle.weight;
            }
        }
    }
    
    if total_weight > 0.0 {
        Some([
            (sum_x / total_weight).round() as i32,
            (sum_y / total_weight).round() as i32,
        ])
    } else {
        None
    }
}
```
**Was macht sie?**
- Gruppiert ähnliche Positionen in Grid-Zellen (3x3)
- Findet die Zelle mit dem höchsten Gesamt-Gewicht
- Berechnet den gewichteten Durchschnitt in dieser Zelle

**Warum Clustering?**
- Partikel sind nicht exakt identisch, sondern "clustern" um die wahre Position
- Ohne Clustering: [10,10], [10,11], [11,10], [9,10] → sehen aus wie 4 Gems
- Mit Clustering: Alles wird zu einer Position ~[10,10]

**Wann aufrufen?**
- Jedes Mal wenn du ein Ziel brauchst

---

## Helper-Funktionen

### Signal-Berechnung
```rust
pub fn calc_signal_strength(distance: f32, signal_radius: f32) -> f32 {
    if distance >= signal_radius {
        0.0
    } else {
        1.0 - (distance / signal_radius)
    }
}
```

### Manhattan Distance
```rust
fn manhattan_distance(a: [i32; 2], b: [i32; 2]) -> f32 {
    ((a[0] - b[0]).abs() + (a[1] - b[1]).abs()) as f32
}
```

### Reverse: Distance für gegebenes Signal
```rust
pub fn calc_dist_for_level(signal_level: f32, signal_radius: f32) -> f32 {
    if signal_level <= 0.0 {
        signal_radius
    } else if signal_level >= 1.0 {
        0.0
    } else {
        (1.0 - signal_level) * signal_radius
    }
}
```

---

## Der komplette Ablauf (jeden Tick)

```rust
// 1. Initialisierung (nur einmal)
if !tracker.initialized {
    tracker.filter.initialize();
    tracker.initialized = true;
}

// 2. Neue Messung hinzufügen
if state.signal_level > 0.0 {
    tracker.filter.update(state.bot, state.signal_level);
    
    // 3. Resample wenn nötig
    let ess = tracker.filter.effective_sample_size();
    if ess < tracker.filter.num_particles as f32 / 2.0 {
        tracker.filter.resample();
    }
    
    // 4. Jitter für Variation
    tracker.filter.jitter();
}

// 5. Hole beste Schätzung
if let Some(target) = tracker.filter.get_best_gem_estimate() {
    // Gehe zu diesem Ziel!
}
```

---

## Wichtige Konzepte nochmal

### Warum funktioniert das?

**Anfang:**
- 1000 zufällige Hypothesen
- Alle gleich wahrscheinlich

**Nach 1. Messung:**
- Hypothesen die passen: hohes Gewicht
- Hypothesen die nicht passen: niedriges Gewicht

**Nach Resample:**
- Gute Hypothesen wurden kopiert
- Schlechte wurden gelöscht
- Alle clustern um "plausible" Positionen

**Nach Jitter:**
- Kleine Variationen → Verfeinerung
- Neue Hypothesen → Exploration

**Nach vielen Messungen:**
- Partikel konvergieren zur wahren Position
- Je mehr Messungen, desto genauer

---

## Debugging-Tipps

### 1. Logge die Partikel-Verteilung
```rust
eprintln!("Top 5 Partikel:");
let mut sorted = tracker.filter.particles.clone();
sorted.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
for (i, p) in sorted.iter().take(5).enumerate() {
    eprintln!("  #{}: weight={:.4}, gems={:?}", i, p.weight, 
        p.gems.iter().map(|g| g.position).collect::<Vec<_>>());
}
```

### 2. Visualisiere Partikel
```rust
// Zeige wo die Top-Partikel ihre Gems vermuten
for particle in tracker.filter.particles.iter().take(20) {
    for gem in &particle.gems {
        let alpha = (particle.weight * 1000.0) as u8;
        highlights.add_point(gem.position, [255, 0, 0, alpha]);
    }
}
```

### 3. Vergleiche expected vs observed
```rust
let expected = particle.calc_expected_signal(state.bot, signal_radius);
eprintln!("Expected: {:.3}, Observed: {:.3}, Diff: {:.3}", 
    expected, state.signal_level, (expected - state.signal_level).abs());
```

---

## Häufige Probleme

### Problem: Alle Partikel haben gleiches Gewicht
**Ursache:** Sigma zu groß
**Lösung:** Verkleinere sigma von 0.1 auf 0.05

### Problem: Nur 1 Partikel hat Gewicht > 0
**Ursache:** Sigma zu klein
**Lösung:** Vergrößere sigma auf 0.15

### Problem: Filter findet keine Gems
**Ursache:** Zu wenig Messungen
**Lösung:** Laufe mehr herum bevor du zum Ziel gehst

### Problem: Filter springt zwischen Positionen
**Ursache:** Zu wenig Partikel oder zu viel Jitter
**Lösung:** Erhöhe num_particles oder verkleinere Jitter-Range

---

## Zusammenfassung: Was brauchst du?

**Structs:**
1. `GemHypothesis` - Eine Position
2. `Particle` - Eine Welt mit 1-3 Gems + Gewicht
3. `ParticleFilter` - Verwaltet alle Partikel

**Funktionen:**
1. `initialize()` - Erstelle zufällige Partikel
2. `update()` - Bewerte Partikel anhand Messung
3. `resample()` - Entferne schlechte, kopiere gute
4. `jitter()` - Füge Variation hinzu
5. `get_best_gem_estimate()` - Hole Ergebnis

**Ablauf:**
```
initialize → (update → resample → jitter)* → get_estimate
```

Das wars! 🎉