import FormalVerification
open EffortSpeed
example : cappedTail 0 3 = 0 := by norm_num [cappedTail]
