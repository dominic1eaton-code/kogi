

* portfolio
    * component
        - metadata
        - id
        - name
        - version
        - timestamp
            - created
            - updated
        - status
        - tags
        - labels
        - users
        - type: item|container

        * container
          - type: bool|binder|archive|folder|dossier|custom

          * item
              - type: resource|project|program|portfolio|asset|artifacts|contact|capital|investment|solution|entity|custom

---

ui components:
dashboard - portfolio overview
portfolio - portfolio manager
wallet - portfolio finances
office - portfolio execution
spsaces - portfolio spaces, portfolio community|shared spaces
market - portfolio resources
hub - portfolio governance
assistant - portfolio assistance, agents

---


kogi - an operating system to give independent workers everything that they need to manage their portfolio's of work:

read the portfolio docs in C:\dev\ws\kogi_dev\kogi-docs\md AND the portfolio code in C:\dev\ws\kogi_dev\kogi-modules\office\src and implement the kogi-portfolio system + master portfolio (hyper)spreadsheet system in C:\dev\ws\kogi_dev\kogi-portfolio. The kogi portfolio+hyperspreadsheet system is implemented using apapo and hypergrid. The kogi master portfolio spreadsheet is the root domain+component that manages+administers+version-controls+controls+maintains all of the portfolio data in the platform, where portfolios contain everything for the work system of an independent worker/organization such as: 

home
    - portable benefits
office
studio 

marketplace
    - work
        - gigs
        - contracts
        - jobs
        - tasks
        - investments
        - offers
        - deals
        - bids
        - requests
        - proposals
        - listings
    - actions
        - rate
        - review
        - match
        - campaign
        - funnels
        - fundraise, resource gather
        - trade
        - allocate
        - fund
        - donate
        - invest
        - contribute
        - find, find talent  (labor, skills, etc...), find resources, find portfolios (and portfolio components (programs, projects, assets, etc...))
    - equity crowdfunding, group economics, regulation crowdfunding, schedule forms/document management system, analytics + capitalization rates/tables + investment metrics

community
    - feed
    - timeline
    - spaces
    - chat
    - message
    - rooms
    - resource sharing, resource access, resource shared economics

exchange
    - exchange: portfolios, skills, resources, labors, workers, gigs, tasks, jobs, deals, etc...
    - equity crowdfunding, group economics
    - financial instruments exchange, liquiity, equity, portfolio assets
    - portfolio commodities exchange, items, goods, services, products, capital, artifacts
    - portfolio resources echange
    - resources+item+users matching

center
    - organizations
        - autonmous orgs
        - collectives
        - cooperatives
        - federations
bank
    - wallet
        - accounts
            - ledgers
            - journals
            - balances
            - status
        - payments
        - taxes
        - orders
        - invoices
        - transactions

developer
    - api
        - internal
        - external
    - sdk
        - internal
        - external
profiles
    - profile.type
        - personal
        - work
        - professional
        - private
        - public
        - custom
    - persona
configuration
    - settings
    - parameters
    - options
    - policies

kogi independent workers:
- contractor
- consultant
- gig worker
- freelancer
- coach
- hobbyist
- enthusiast
- entreprenuer
- micropreneur

organization:
- autonomous organization
- cooperative
- collective
- federation
- team
- group
- cell

user:
- editor
- owner
- collaborator
- member

personas:
- creator
- professional
- developer
- donor
- investor
- contributor
- partner
- organizer
- facilitator
- visionary
- integrator

action:
- like
- subscribe
- follow
- donate
- invest
- comment
- post
- watch
- bookmark
- friend
- connect
- share
- tag
- edit
- view

containers:
- record
- book
- note
- memo
- binder
- archive
- record
- dossier
- folder
- schedule
- directory
- calendar
- gantt
- roadmap
- timebox
- wallet
- account
- room
- chat
- graph
- matrix
- grid
- workspace
- namespace
- toolbox
- linknet
- linktree
- linkforest
- form
- resume
- backlog

items:
- solution
- resource
- asset
- artifact
- investment
- entity
- estate
- real estate
- capital
- land
- labor
- skill
- knowledge
- contact
- charter
- document
- file
- work package
- project
- program
- portfolio
- gig
- contract
- consultation
- meeting
- appointment
- conference
- space
- listing
- booking
- benefit
- security
- liquidity
- cash
- credit
- debt
- tax
- invoice
- offer
- deal
- bid
- request
- proposal
- post
- ideas
- tool
- toolkit
- toolset
- toolchain
- link
- agreement

portfolio.component
portfolio.item
portolio.task
portfolio.executor

item.binder
item.book
    book.charter
        - brief
        - description
        - assumptions
        - risks
        - goals
        - milestones
        - objectives
        - outcomes
item.attachment[s]
item.linktree
    linktree.account
    linktree.integration
item.owner[s]
item.data
item.metadata
item.space


work management system

- workspace
    - work dashboard
    - work backlogs + backloags management system
    - work governance
    - work content management system
        - files
        - documents
        - contracts
        - agreements
        - SOPs
        - policies
        - procedures
        - frameworks
        - models
    - work boards
    - work timelines
        - schedules
        - gantts
        - calendars
        - roadmaps
        - timeboxes: program incements PIs, sprints, custom timeboxes, durations, qaurters
    - work analytics
        - forecasting
        - analysis
        - telemetry
        - optimization
        - personalization
        - performance
        - KPIs
        - OKRs
        - data tracking
    - work resource management
        - budgeting
        - reporting
        - allocation
        - delegation
        - TODO's
          - do now
          - do later
          - delegate
          - marked for deletion
    - work studio
        - requirements management system
        - work design systems
- portfolio.components
  - portfolio
  - program
  - project
  - resource
    - knowledge; skills
    - time
      - schedule
      - timeline
      - roadmap
      - timebox
      - time epoch
        - quarter
        - sprint
        - PI
        - cycle
    - contacts
    - capital
    - labor
    - budget; provision; allocation
    - finance+liquidity+equity+securities; accounts - wallets
  - asset
    - solution, product, service, good, platform, application, investment, estate, real estate, devices/hardware
  - artifact
    - documents, files, archives, outcomes, deliverables, plans, reports, charters, registers, risk registers, project plans, status reports, project charters, etc...
- work breakdown structure WBS; portfolio.items
    - work package
        - theme
            - initiative
                - epic
                    - story
                        - task
                    - story.data:
                        + owners
                        + unique id
                        + name
                        + labels
                        + categories
                        + classes
                        + types
                        + dependencies
                        + dependents
                        + children
                        + parents
                        + attachments
                        + fields
                        + timestamps (creation, update)
                        + tags
                    - story.type:
                        + feature
                        + bug
                        + testing
                        + capability
                        + issue
                        + defect
                        + enhancement
                        + innovation
                        + audit
                        + enabler
                        + blocker
                        + use case
                        + business case
                        + requirement
                        + documentation
                        + milestone
                        + goal
                        + objective
                        + outcome
                        + mission
                        + vision
                        + risk
                        + analysis
                        + strategy
                        + tactic
                        + operation
                        + plan
                        + report
                        + release
                        + deployment
                        + distribution
                        + template
                        + archive
                        + gig
                        + job
                        + contract
                        + consultation
                        + booking
                        + meeting
                        + appointment


identity management system

- multiple user identities management
- users management system
- sessions management system
- profiles management system
- personas, roles management system
- contact management system
- access control system


accounts management system

user accounts
    - personal accounts
    - work accounts
    - professional accounts
    - burner accounts
    - AI+agent+automation accounts
service+AI accounts

profile management system

users can have different types of profiles, that have different accounts associated with them:
- work profiles
- professional profiless
- personal profile
- public profiles
- private profiles
- protected profiles
- business profiles
- miscellaneaous profiles
- custom profiles
- template profiles

a linkforest is a connected collection, a forest, of linktrees, where a linktree is a connected chain (linked list) of linked accounts, all forest+trees+accounts connected by a root datastructure substrate called a "linknetwork"

kogi linktree
    - user's personal IP address / router, link to outside world + external tools + contacts
    - kogi master portfolio spreadsheet runs and links thourgh the linktree
    - linktrees connecting/connected through linknodes, form linkforests, which sets on top of a connected linknetwork


content creation portfolio management

integrations+connections+vendors

portfolio
- google workspace+account
- google sheets
- excel sheets
- notion
- jira
- monday
- gitlab
- github
- bitbucket
- yahoo account
- microsoft account
- microsoft office
- asana
- calendly
- clickup
- gohighlevel
- soundcloud
- infusionsoft; keap
- clickfunnels
- ontraport
- servicenow
- coda
- obdisdian
- motion
- airtable
- figma
- confluence
- clickify
- trello
- dropbox
- evernote
- zoho
- odoo
- salesforce
- GA4
- pipedrive
- akiflow
- obsidian
- sansama
- mayday
- toggl track
- superhuman
- ideashell
- anytype
- craft
- verilyme
- stride health
- shiftmate
- alia health
- Linear
- Mercury (banking)
- Deel (contractor payments)

wallet
- wellsfargo
- bank of america
- stripe
- venmo
- klarna
- paypal
- relayfi
- coinbase
- ethereum
- mpesa
- quickbooks
- cashapp
- wise
- robinhood
- zelle
- apple cash
- googlepay
- payoneer
- square
- startengine
- patreon
- gofundme
- wefunder
- indiegogo
- kickstarter
- freshbooks
- soci.ai


marketplace
- amazon
- ebay
- craigslist
- behance
- LTK
- etsy
- shopify
- shopmy
- beacons
- mavely
- stylmatch
- dribbble
- wix
- squarespace
- ghost
- mightynetworks
- kajabi
- wordpress
- skool
- kartra
- circle
- uber
- lyft
- instacart
- shipt
- taskrabbit
- fiverr
- upwork
- freelancer.com
- peopleperhour
- guru
- toptal
- gigsmart
- thumbtack
- angies list
- yelp
- rover
- wag!
- clickworker
- shiftsmart
- handy
- porch
- nextdoor
- care.com
- wonolo
- airtasker
- airbnb
- usertesting
- docusign
- bidsketch
- eversign
- hubspot
- mailchimp
- jobber
- housecallpro
- zendesk
- app.cal.com
- scribe

spaces|community
- slack
- discord
- meetup
- whatsapp
- facebook
- facebook messenger
- LinkedIn
- skool
- youtube
- X/twitter
- bluesky
- mastadon
- twitch
- groupme
- kik
- telegram
- snapchat
- instagram
- tiktok
- fanbase
- onlyfans
- justforfans
- meetup
- eventbrite
- linktree
- substack
- reddit
- threads
- quora
- stackk overflow
- pinterest
- vimeo
- ghost
- spotify
- apple music+podcast
- google hangouts+meets
- zoom
- skype
- community.com


assistant
- oba
- openai
- chatGPT
- claude
- grok
- deepseek
- mistral
- llama
- perplexity
- poe
- gemini
- cursor
- windsurf
- codex
- saner.ai
- zapier
- integrately
- castmagic
- luvable
- replit
- base44
- heyjen
- bolt.new
- bubble
- v0
- magicpatterns
- orchids
- firebase studio
- reclaim.ai

---

publication channels
- marketplace
- community
- exchange
- rooms
- spaces
- private|public|protected

IP management system
    - patents
    - rights
    - copyrights
    - trademarks
    - watermarks
    - licenses
    - branding, logos, marks
    - contracts, agreements


Resource Management System
- portfolios, programs, projects, assets, artifacts, binders, journals, books, dossiers, folders, documents, files, directories, 
- timelines, schedules, roadmaps, calendars, gantts
- boards, stories, epics, features, stories, tasks, work packages, work breakdown structures, initiatives, strategies, tactics, operations, themes
- gigs, consultations, bookings, tasks, jobs, contracts, offers, deals, requests, proposals, bids, investments, gigs, orders, listings
- capital, assets, artifacts, labor, land, estates, real estate
- equitty, liquidity, debt, taxes, cash, credit, debit, donations, grants
- contributions (labor, skills, financial, support, advertising, marketing, promoions, endorsements, donation, investment)
- users: member, contributors, donors, investors, subscribers, followers, watcher, owner, editor
- personas:  developers, creatives, artists, writers, journalists, professionals, enthusiasts, hobbyists, service providers, visionaries, architects, designers, facilitators, integrators, organizers, activists, managers, directors, insiders, hackers, technicians, innovators, technologists
- workers: contractors, consultants, gig workers, freelancers, entreprenuers, micropreneurs, coaches, partners, employees, officers
- organizations: autonomous|independent|ad-hoc organizations, open source communities, cooperatives, collectives, federations, autonomous|independent|ad-hoc teams|groups, councils, assemblies


communication channels
email
message - unicast (direct), multicast (group), broadcast [alerts, notifications, announcements]
notifications, alerts, announcements
communication channels:
messaging
email
phone
voice call
video call

rooms
- rooms: gigs, consultations, bookings, tasks, jobs, contracts, offers, deals, requests, proposals, bids, investments, gigs, orders, listings
- communinity+chat rooms
- rooms: organizations, collectives, cooperatives, groups, teams, federations, one on one

spaces
communities, marketplaces, groups, teams, independent worker, exchanges

