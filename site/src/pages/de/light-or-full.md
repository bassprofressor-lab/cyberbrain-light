---
layout: ../../layouts/Doc.astro
title: "Light oder voll"
description: "Wann Cyberbrain Light reicht und wann das volle Cyberbrain mit Bedeutungssuche, Widerspruchsprüfung und DSGVO-/KI-Verordnungs-Nachweisen besser passt."
lang: de
slug: "light-or-full/"
---

# Light oder voll

|  | Light | [Cyberbrain](https://github.com/bassprofressor-lab/cyberbrain) |
|---|---|---|
| Suche | Stichwörter (BM25) | Stichwörter **und** Bedeutung, kombiniert |
| Modell | keins | ein Einbettungsmodell, das du selbst ablegst |
| Speicher beim Suchen | 8 MB | 1,55 GB |
| Widerspruchsprüfung | nein | gegen einen lokalen Inferenzdienst |
| Nachweise für DSGVO und KI-Verordnung | nein | Löschung, Auskunft, Audit-Kette, PII-Prüfung, Pflichten |
| Weboberfläche | nein | im Programm eingebaut, Deutsch und Englisch |
| Programm | 5,0 MB | 13,6 MB |

**Light passt**, wenn eine Person oder ein kleines Team will, dass der Agent sich Projektwissen merkt, und in den Notizen keine personenbezogenen Daten stehen: Bauschritte, Entscheidungen, Fehler und ihre Lösung. Es läuft überall, kostet im Betrieb nichts und kann nichts über das Netz verlieren.

**Das volle Cyberbrain passt**, wenn die Suche Bedeutung finden soll und nicht nur Wörter, oder wenn im Store personenbezogene Daten liegen können und du zeigen musst, was gespeichert wurde, wer es geschrieben hat und dass eine Löschung wirklich passiert ist. Dafür gibt es Audit-Kette, PII-Prüfung und Löschnachweise.

> Beide können denselben Store nutzen. Light führt die Prüfungen des vollen Produkts aber nicht aus: Eine über Light geschriebene Notiz umgeht PII-Prüfung und Audit-Eintrag, und `cbl forget` hinterlässt keinen Löschnachweis. In einem Store, der darauf angewiesen ist, nur über `cyberbrain` schreiben.

[Cyberbrain auf krynexlabs.de](https://www.krynexlabs.de/cyberbrain/)
