Bei einem Gem ist der Gendanke relativ einfach:
Wenn er also das Signal gesehen wird, dann wird geguckt in welchem Radius er sich vom Spieler befindet. Dann werden die verschiedenen Orte auch mit anbetracht der Werte von vorherigen Ticks bewertet und dann zum dem Wahrscheinlichstem Ort geganen. 

Sind mehrere Gems vorhanden, werden alle möglichen Positionen der Gems (es gibt maximal 3) angenommen und bewertet, ob sie zu dem aktuellen Signal und zu dem Signal, welches in den letzten Ticks gesehen wurde passt. 
Das kann mit dem Timelimit Probleme geben: bei einer Map von 49x29 sind das 1420 Felder. Mit 3 Gems gibt es ca. 3 Milliarden (wenn ich kein Mathe verlernt habe) Positionen für die Gems. Das kann aber durch das Einbeziehen von walls oder auch von der Signal stärke, da diese ja auch nicht über die gesamte Map geht, einbegrenz werden. Dazu müssen auch noch die walls welche nicht gesehen werden also sich bspw. in anderen walls befinden auch in known walls, oder weil sie ja eigentlich nicht duchgehend gebraucht werden, ein einem anderen Objekt

Man könnte jeden Tick die Signal stärke mit dem Zeitpunkt speichern, und somit, wenn die Info sehr alt ist, ihr weniger Wert zuschreiben als eine aktuellere Beobachtung. 

Basicly der metropolis hastings algorythm