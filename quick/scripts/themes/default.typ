// Theme: default
// Accent-blue construction spec theme with DRAFT watermark and full header/footer chrome.
// This is the original built-in theme for the quick/ bilingual pipeline.
#let accent  = rgb("#1a6b8a")
#let light   = rgb("#f0f7fa")
#let muted   = rgb("#666666")

// Two-column image grid helper.
// Use in a raw typst block inside any spec .md file:
//
//   ```{=typst}
//   #grid-images(
//     image("images/gate/before.jpg"),
//     image("images/gate/after.jpg"),
//   )
//   ```
#let grid-images(..imgs) = {
  v(0.3em)
  align(center, block(width: 100%,
    grid(
      columns: (1fr, 1fr),
      gutter: 0.5em,
      ..imgs.pos().map(img =>
        block(stroke: 0.5pt + rgb("#dddddd"), radius: 3pt, clip: true)[#img]
      )
    )
  ))
  v(0.3em)
}

// Captioned image — used by cmarker for every markdown image.
// Renders the image with a caption underneath: "Figure N — <alt> · <filename>"
// so each image in the PDF has both human-readable label and file provenance.
#let captioned-image(src, alt: none, ..args) = {
  let has-alt = alt != none and str(alt).trim() != ""
  figure(
    image(src, ..args),
    caption: {
      set text(size: 8pt)
      if has-alt {
        emph(alt)
        h(0.4em)
        text(fill: rgb("#888888"))[· #raw(src)]
      } else {
        text(fill: rgb("#888888"))[#raw(src)]
      }
    },
  )
}

#let project-name = "Laem Chabang House"
#let build-date   = datetime.today().display("[day] [month repr:long] [year]")

#let conf(
  title: none,
  subtitle: none,
  authors: (),
  keywords: (),
  date: none,
  abstract-title: none,
  abstract: none,
  thanks: none,
  cols: 1,
  margin: (x: 1.8cm, top: 2.0cm, bottom: 2.4cm),
  paper: "a4",
  lang: "en",
  region: "US",
  font: none,
  fontsize: 10pt,
  mathfont: none,
  codefont: none,
  linestretch: 1.4,
  sectionnumbering: none,
  linkcolor: none,
  citecolor: none,
  filecolor: none,
  pagenumbering: "1",
  status: "Draft",
  rev: "1",
  doc,
) = {

  set document(
    title: if title != none { title } else { project-name },
    author: project-name,
  )

  set page(
    paper: paper,
    margin: margin,
    header: context {
      if counter(page).get().first() > 1 {
        set text(size: 8pt, fill: muted)
        grid(
          columns: (1fr, 1fr),
          align(left)[#project-name — #title],
          align(right)[#build-date],
        )
        line(length: 100%, stroke: 0.4pt + muted)
      }
    },
    footer: context {
      set text(size: 8pt, fill: muted)
      line(length: 100%, stroke: 0.4pt + muted)
      grid(
        columns: (1fr, 1fr, 1fr),
        align(left)[#title#".pdf"],
        align(center)[Rev #rev — #status],
        align(right)[#counter(page).display("1 / 1", both: true)],
      )
    },
    background: context {
      if status == "Draft" {
        place(center + horizon,
          rotate(-45deg,
            text(
              size: 96pt,
              weight: "bold",
              fill: rgb("#ebebeb"),
            )[DRAFT]
          )
        )
      }
    },
  )

  set text(
    font: ("Inter", "Noto Sans", "Noto Sans Thai"),
    size: fontsize,
    lang: lang,
    hyphenate: false,
  )

  set par(leading: 0.65em, spacing: 1.0em)

  if title != none {
    block(width: 100%, fill: accent, inset: (x: 1em, y: 0.6em), radius: 4pt)[
      #grid(
        columns: (1fr, auto),
        align(left + horizon)[
          #text(size: 20pt, weight: "bold", fill: white)[#title]
        ],
        align(right + horizon)[
          #set text(size: 8pt, fill: white.transparentize(20%))
          #project-name \
          #build-date \
          Rev #rev — #status
        ],
      )
    ]
    v(0.8em)
  }

  show heading.where(level: 1): it => {
    pagebreak(weak: true)
    v(0.6em)
    block(width: 100%, fill: light, inset: (x: 0.7em, y: 0.45em), radius: 3pt)[
      #text(size: 12pt, weight: "bold", fill: accent)[#it.body]
    ]
    v(0.3em)
  }

  show heading.where(level: 2): it => {
    v(0.5em)
    text(size: 10.5pt, weight: "bold", fill: accent)[#it.body]
    line(length: 100%, stroke: 1.5pt + accent)
    v(0.25em)
  }

  show heading.where(level: 3): it => {
    v(0.4em)
    text(size: 10pt, weight: "bold", fill: muted)[#it.body]
    v(0.15em)
  }

  set table(
    inset: (x: 0.5em, y: 0.35em),
    stroke: (_, y) => if y == 0 { none } else { (top: 0.4pt + rgb("#e0e0e0")) },
    fill: (_, y) => if y == 0 { accent } else if calc.odd(y) { rgb("#f8f8f8") } else { white },
  )
  show table.cell: set text(size: 8.5pt)
  show table.cell.where(y: 0): set text(weight: "bold", fill: white, size: 8.5pt)
  show table: set block(width: 100%)

  show link: set text(fill: accent)

  show figure: it => {
    v(0.3em)
    align(center, block(width: 100%)[
      #block(stroke: 0.5pt + rgb("#dddddd"), radius: 3pt, clip: true)[#it.body]
      #if it.caption != none [
        #v(0.25em)
        #set par(leading: 0.4em)
        #it.caption
      ]
    ])
    v(0.5em)
  }

  show line: set line(stroke: 0.5pt + rgb("#dddddd"))

  doc
}
