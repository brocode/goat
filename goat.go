package main

import ui "github.com/gizak/termui"
import flags "github.com/jessevdk/go-flags"
import (
	"fmt"
	"math"
	"os"
	"time"
	"strings"
	"strconv"
)

type opts struct {
	Time  int    `short:"t" long:"time" description:"timer in seconds" required:"true"`
	Title string `long:"title" description:"title"`
	Mappings []string `short:"m" long:"mapping" description:"Keybinding mapping. Format: <retcode>:<key>:<label> (64 <= retcode <= 113)"`
}

type mapping struct {
	exitCode int
	label string
	key string
}

func parseMappings(rawMappings []string) ([]mapping, error) {
	var mappings []mapping
	for _, rawMapping := range rawMappings {
		var slicedMapping =	strings.Split(rawMapping, ":")
		if(len(slicedMapping) != 3){
			return nil, fmt.Errorf("Invalid mapping '%s', format should be <retcode>:<key>:<label>", rawMapping)
		}
		var exitCode, err = strconv.Atoi(slicedMapping[0])
		if err != nil || exitCode < 64 || exitCode > 113 {
			return nil, fmt.Errorf("Invalid mapping '%s', retcode '%s' is either not a number or < 64 or > 113", rawMapping, slicedMapping[0])
		}

		mappings = append(mappings, mapping {exitCode: exitCode, key: slicedMapping[1], label: slicedMapping[2] })
	}
	return mappings, nil
}

func main() {
	var opts = opts{}

	if _, err := flags.Parse(&opts); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	mappings, err := parseMappings(opts.Mappings)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}


	os.Exit(runUI(opts, mappings))
}

func timerGauge() *ui.Gauge {
	timerGauge := ui.NewGauge()
	timerGauge.Percent = 0
	timerGauge.Width = 50
	timerGauge.Height = 5
	timerGauge.Y = 6
	timerGauge.BorderLabel = "Timer"
	timerGauge.PercentColor = ui.ColorYellow
	timerGauge.BarColor = ui.ColorGreen
	timerGauge.BorderFg = ui.ColorWhite
	timerGauge.BorderLabelFg = ui.ColorMagenta
	return timerGauge
}

func headerBox(title string, mappings []mapping) *ui.Par {
	var shortcuts []string
	shortcuts = append(shortcuts, "'q' -> abort")
	shortcuts = append(shortcuts,	"'c' -> continue")
	for _, mapping := range mappings {
		shortcuts = append(shortcuts, fmt.Sprintf("'%s' -> %s", mapping.key, mapping.label))
	}

	keyText := fmt.Sprintf("KEYBINDINGS:\n%s", strings.Join(shortcuts, "\n"))
	var boxTitle string
	if title != "" {
		boxTitle = title
	} else {
		boxTitle = "goat"
	}
	p := ui.NewPar(keyText)
	p.Height = 5 + len(mappings)
	p.Width = 50
	p.TextFgColor = ui.ColorBlack
	p.BorderLabel = boxTitle
	p.BorderFg = ui.ColorCyan
	return p
}

func runUI(opts opts, mappings []mapping) int {
	returnCode := 0
	if err := ui.Init(); err != nil {
		panic(err)
	}

	start := time.Now()

	p := headerBox(opts.Title, mappings)

	timerGauge := timerGauge()

	ui.Body.AddRows(
		ui.NewRow(
			ui.NewCol(12, 0, p),
		),
		ui.NewRow(
			ui.NewCol(12, 0, timerGauge)))

	// calculate layout
	ui.Body.Align()

	ui.Render(ui.Body)

	ui.Handle("/sys/kbd/q", func(ui.Event) {
		returnCode = 1
		ui.StopLoop()
	})
	ui.Handle("/sys/kbd/c", func(ui.Event) {
		ui.StopLoop()
	})

	for _, mapping := range mappings {
		exitCode := mapping.exitCode
		ui.Handle(fmt.Sprintf("/sys/kbd/%s", mapping.key), func(ui.Event) {
			returnCode = exitCode
			ui.StopLoop()
		})
	}

	// this does not work for some reason
	// ui.Handle("/sys/wnd/resize", func(e ui.Event) {
	// 	ui.Body.Align()
	// 	ui.Render(ui.Body)
	// })

	// handle a 1s timer
	ui.Handle("/timer/1s", func(e ui.Event) {
		duration := time.Since(start)
		if int(duration.Seconds()) >= opts.Time {
			ui.StopLoop()
		} else {
			timerGauge.Percent = int(math.Min(duration.Seconds()/(float64(opts.Time))*100, 100))
			ui.Body.Align()
			ui.Render(ui.Body)
		}
	})

	ui.Loop()
	ui.Close()
	return returnCode
}
