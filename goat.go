package main

import ui "github.com/gizak/termui"
import flags "github.com/jessevdk/go-flags"
import (
	"fmt"
	"math"
	"os"
	"time"
)

func main() {

	var opts struct {
		Time int `short:"t" long:"time" description:"timer in seconds" required:"true"`
	}

	returnCode := 0

	_, err := flags.Parse(&opts)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	if err := ui.Init(); err != nil {
		panic(err)
	}

	start := time.Now()

	p := ui.NewPar("'q' TO ABORT\n'c' TO CHECK NOW")
	p.Height = 5
	p.Width = 50
	p.TextFgColor = ui.ColorBlack
	p.BorderLabel = "goat"
	p.BorderFg = ui.ColorCyan

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
	os.Exit(returnCode)
}
