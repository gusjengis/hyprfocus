# Todo
- [ ] Better Usage/Help text
- [ ] Arbitrary Interval Reports
- [ ] Handle table formatting for long durations and extra wide characters
- [ ] Output all errors to a log file (~/.local/share/hyprfocus/hyprfocus.log)
- [ ] Generate notification for all logs for debug purposes
- [ ] Implement settings and args for cutoffs for long reports

### Configuration
- [ ] Setup hyprfocus.conf and get parsing working
    - [ ] display settings
        - [ ] fancy_timeline
        - [ ] characters
        - [ ] colors
        - [ ] layout
        - [ ] minimum duration to display
        - [ ] show labels next to timelines in multi
    - [ ] default argument settings
    - [ ] ignore settings
    - [ ] category settings
    - [ ] map classes to other classes using regex Ex: steam_app_{number} -> steam

- [ ] If a idle is logged and something shows up before the resume signal, write resume anyway?

### Unsolved bugs
- [ ] Figure out why logs stopped showing up while the daemon was still running on 8/5/25
- [ ] Played ANIMAL WELL for a while, it showed up on the timeline, but the report only said 10 minutes
        1754519405730,steam_app_813230,"ANIMAL WELL"
        1754519870018,SYSTEM,"idle"
        1754525658737,kitty,"vim"
        I was using a controller, but the idle came well after 3 minutes
        And there was no resume, all very strange

        This may have something to do with hypridle, for some reason I never
        got the resume signal, and apparently hypridle isn't smart enough to check
        controller use
