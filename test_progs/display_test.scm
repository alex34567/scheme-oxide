;    Copyright 2019 Alexander Eckhart
;
;    This file is part of scheme-oxide.
;
;    Scheme-oxide is free software: you can redistribute it and/or modify
;    it under the terms of the GNU General Public License as published by
;    the Free Software Foundation, either version 3 of the License, or
;    (at your option) any later version.
;
;    Scheme-oxide is distributed in the hope that it will be useful,
;    but WITHOUT ANY WARRANTY; without even the implied warranty of
;    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;    GNU General Public License for more details.
;
;    You should have received a copy of the GNU General Public License
;    along with scheme-oxide.  If not, see <https://www.gnu.org/licenses/>.

(display (string-ref "A" 0))
(newline)
(display 0)
(newline)
(display 8747835)
(newline)
(display "Hello World")
(newline)
(display '())
(newline)
(display '("Hello" "World" 9))
(newline)
(display '(("Nesting")))
(newline)
(display '("Improper" . "List"))
(newline)
(display #f)
(newline)
(display '''Test)
(newline)
(display (eqv? 'test 'test))
(newline)
