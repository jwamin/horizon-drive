# Horizon Drive — mechanics sheet

An original arcade road racer in the style of the target (sunny checkpoint racer with branching routes).
**Clean room:** nothing inside the original program was read, and no pictures or captures of it were supplied.
So **every number below is estimated** from the genre and tuned by playtesting with the autopilot.
The code uses these exact values (`src/consts.rs`); `mechanics.json` holds the same list.

## Controls (estimated)
| Action | Keys |
|---|---|
| Accelerate | Up / W / X |
| Brake | Down / S / Z / Space |
| Steer | Left/Right or A/D |
| Drift | Tap brake (shorter than 0.30 s) while steering above 110 km/h |
| Toggle automatic/manual | M |
| Gear up / down (manual) | E or Left Shift / Q or Left Ctrl |
| Pause | Esc or P |
| Menus | Arrows + Enter |

## Course
| Value | Number | Source |
|---|---|---|
| Stages per run | 5 (pyramid of 15 stages: 1-2-3-4-5) | estimated (genre) |
| Stage length | 4.4 km (880 segments x 5 m) | estimated |
| Fork zone | last 600 m of a stage; branches end 28 m either side of the old centre | estimated |
| Road | 4 lanes, 16 m wide, 1.2 m rumble strips | estimated |
| Tightest bend | curve 0.024 per segment² | estimated |
| Hills | up to ±28 m per section, ±60 m overall | estimated |
| Draw distance | 1.3 km | estimated |

## Camera
| Value | Number | Source |
|---|---|---|
| Height above road | 2.8 m | estimated |
| Distance behind car | 6.5 m | estimated |
| Vertical field of view | 50° | estimated |
| Horizon | 42% down the screen | estimated |
| Sideways follow rate | 6 /s | estimated |

## Timer and rules (Arcade Run)
| Value | Number | Source |
|---|---|---|
| Starting time | 75 s | estimated |
| Time added at checkpoints 1-4 | +70, +68, +66, +64 s | estimated |
| Countdown | 3 s | estimated |
| Time runs out | car coasts to a stop, GAME OVER | estimated |
| Goal | 5th stage end; one of 5 goals depending on route | estimated (genre) |

## Driving (per car)
| Car | Top speed | Base acceleration | Steering |
|---|---|---|---|
| Rossa Barchetta (balanced) | 290 km/h | 30 km/h/s | 10 m/s sideways |
| Vento Speedster | 312 km/h | 26 km/h/s | 9 m/s |
| Piccola Roadster | 272 km/h | 35 km/h/s | 11.5 m/s |

All estimated. Shared values:

| Value | Number | Source |
|---|---|---|
| Acceleration curve | base × (1 − (v/top)³), never below 12% until top speed | estimated |
| Gears | 6; each tops out at 24/41/59/74/88/100% of top speed | estimated |
| Automatic shifts up at | 96% of the gear's top | estimated |
| Manual: gear too high for speed | 55% pull | estimated |
| Shift time | 0.15 s (no drive) | estimated |
| Off-throttle slowdown | 9 km/h per s | estimated |
| Brakes | 95 km/h per s | estimated |
| Off road | slows to 110 km/h at 75 km/h per s | estimated |
| Steering builds to full by | 80 km/h | estimated |
| Steering response | 8 /s | estimated |
| Bend push (outward) | 13 m/s in the tightest bend at top speed (scales with speed²) | estimated |

Result (measured by the self-test): about 0-100 km/h in 4 s, 160 km/h after 9 s; the autopilot clears stage 1 in about 1:04.

## Drift
| Value | Number | Source |
|---|---|---|
| Start | brake tap < 0.30 s while steering, above 110 km/h | estimated |
| Keeps going above | 85 km/h | estimated |
| Ends | 0.25 s without steering into it | estimated |
| Speed loss while drifting | 6 km/h per s | estimated |
| Steering while drifting | ×1.35 | estimated |
| Bend push while drifting | ×0.55 | estimated |
| Visual car angle | up to 0.45 rad | estimated |

## Collisions
| Value | Number | Source |
|---|---|---|
| Hitting scenery | spin-out, 2.5 s, restart from standstill on nearest lane | estimated |
| Rear-ending traffic | speed drops to 75% of their speed, pushed 1.4 m sideways | estimated |
| Hitting traffic 140+ km/h faster | spin-out | estimated |

## Traffic
| Value | Number | Source |
|---|---|---|
| Cars per stage | 34 × stage density (0.7-1.3) | estimated |
| Speeds | 70-140 km/h (trucks and buses ×0.8) | estimated |
| Mix | sedans 40%, hatchbacks 20%, vans 20%, trucks 10%, buses 10% | estimated |
| Behaviour | keep their lane, no traffic in the fork zone | estimated |

## Score
| Value | Number | Source |
|---|---|---|
| Driving | 3 points per km/h per second | estimated |
| Overtake | 500 | estimated |
| Drifting | 300 per second | estimated |
| Checkpoint | 20,000 | estimated |
| Goal time bonus | 10,000 per second left | estimated |
| Passenger task done | 50,000 | estimated |

## Modes
* **Arcade Run** — the rules above.
* **Time Attack** — no traffic, no countdown limit; clock counts up; best time per goal is kept for the session. (estimated)
* **Passenger Challenge** — same timer as Arcade; each stage the passenger sets a task; meeting it earns a heart (5 max), rank E…AAA at the end. (estimated)
  Tasks rotate by stage: overtake 10+2×row cars; drift 250+80×row m; pop 10+2×row balloons (25 placed); stay over 200 km/h for 14+3×row s; don't hit anything.

## On-screen information
Score (top left) with stage name and hearts; TIME (top centre, flashes red under 10 s); LAP and TOTAL (top right);
speed in km/h, rev bar and AT/MT gear (bottom right); route map of all 15 stages with your path plus stage progress bar (bottom left).
Messages: countdown, CHECKPOINT +n SEC, CHOOSE YOUR ROUTE, DRIFT!, CRASH!, TIME UP, GOAL!. Layout estimated (no pictures of the original).

## Sound
Three original procedural "radio" tunes (picked before the race), countdown beeps, checkpoint jingle, crash/bump, skid loop. No engine note yet.
