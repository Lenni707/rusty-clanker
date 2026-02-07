Bei einem Gem ist der Gendanke relativ einfach:
Wenn er also das Signal gesehen wird, dann wird geguckt in welchem Radius er sich vom Spieler befindet. Dann werden die verschiedenen Orte auch mit anbetracht der Werte von vorherigen Ticks bewertet und dann zum dem Wahrscheinlichstem Ort geganen. 

Sind mehrere Gems vorhanden, werden alle möglichen Positionen der Gems (es gibt maximal 3) angenommen und bewertet, ob sie zu dem aktuellen Signal und zu dem Signal, welches in den letzten Ticks gesehen wurde passt. 
Das kann mit dem Timelimit Probleme geben: bei einer Map von 49x29 sind das 1420 Felder. Mit 3 Gems gibt es ca. 3 Milliarden (wenn ich kein Mathe verlernt habe) Positionen für die Gems. Das kann aber durch das Einbeziehen von walls oder auch von der Signal stärke, da diese ja auch nicht über die gesamte Map geht, einbegrenz werden. Dazu müssen auch noch die walls welche nicht gesehen werden also sich bspw. in anderen walls befinden auch in known walls, oder weil sie ja eigentlich nicht duchgehend gebraucht werden, ein einem anderen Objekt

Man könnte jeden Tick die Signal stärke mit dem Zeitpunkt speichern, und somit, wenn die Info sehr alt ist, ihr weniger Wert zuschreiben als eine aktuellere Beobachtung. 

Ok, woher wissen wir, wie viele Gems gerade gleichzeitig auf dem Feld sind




Basicly der metropolis hastings algorythm

-- GROBE IDEE --
- 1421 Felder - (PI * VISUAL_DIST^2) - known_walls
- VISUAL_DIST von 5-10

- Für erste position durchrechnen, bis zum Zeitlimit => ein schritt weiter, wieder durchrechnen aber die bereits ausgeschlossenen Positionen aus der ersten berechnung nicht mit einbeziehen

-- PROBLEM --
woher weiß ich wie viele Gems es genau gerade gibt 
=> Lösung: Die anzahl an gems ist variabel, heißt also einfach auch mit der anzahl der gems rechnen, was da am wahrscheinlichsten die position wäre (ergo 1 gem, 2 gems, 3 gems)

-- ANDERE IDEE --

- Algo ähnlich wie oben, nur diesmal mit metropolis hasting
- Heißt:
    - Gem anordnungen ausprobieren (felder aus visual_range und felder aus known walls ignorieren)

    - wenn bessere anordnung gefunden (entspricht am nächsten dem signal werit) => direkt behalten 
    - wenn schlechter => manchmal behalten, zu einer gewichteten chance
    - weitere anordnungen ausprobieren, die der von davor ähneln (einfach wieder von vorne das prozedere) => möglicherweise könnte man auch gems wegnehmen oder dazu adden
    - nach jeder bewegung des bottes müssen die möglichen gem layouts auch mit dem signal level der vorherigen positionen des bots übereinstimmen (vielleicht iwie ein safety net, das werte nicht zu doll springen dürfen oder so, weil dann, höchstwahrscheinlich ein neuer gem gespawnt ist)

    - laufen lassen...


    - am ende hat man ganz viele guesses wo die gems sein können. (so nach 2-3 moves des bottes)
    - heatmap aus feldern je mit werten, wie oft sie in positiven rate versuchen vorkommen
    - zum nähesten Feld mit dem höchsten Wert gehen (höchste wahrscheinlichkeit, dass auf diesem feld ein gem ist) => abwägen risk vs distance (vielleicht nahe felder mit trotzdem herausstechender wahrscheinlichkeit priotisieren)

    Mögliche Probleme des Algos:
    - Neue gems spawnen
    - was macht man, wenn man einen gem gefunden hat. => wie geht man am besten zum nächsten. (vielleicht unsere erste idee mit triliteration aus ausgewählten known positions)    

Nachdem ich Chatgpt meine 2am idee präsentiert habe, hat er mir diesen algorythmus als besser vorgeschlagen:
	•	Particle Filter (Sequential Monte Carlo):