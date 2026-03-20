import { Routes } from '@angular/router';
import { LoginComponent } from './index/login/login.component';
import { RegistrationComponent } from './index/registration/registration.component';
import { OnboardingComponent } from './index/onboarding/onboarding.component';
import { HomeComponent } from './home/home.component';
import { HomeLandingComponent } from './home/home-landing/home-landing.component';
import { HomeAboutComponent } from './home/home-about/home-about.component';
import { HomePlatformComponent } from './home/home-platform/home-platform.component';
import { HomeEcosystemComponent } from './home/home-ecosystem/home-ecosystem.component';
import { CreatingWorkspaceComponent } from './index/creating-workspace/creating-workspace.component';
import { DashboardComponent } from './dashboard/dashboard.component'
import { DashboardOverviewComponent } from './dashboard/dashboard-overview/dashboard-overview.component'
import { DashboardCalendarComponent } from './dashboard/dashboard-calendar/dashboard-calendar.component';
import { PortfolioComponent } from './portfolio/portfolio.component';
import { PortfolioCreateComponent } from './portfolio/portfolio-create/portfolio-create.component';
import { PortfolioDashboardComponent } from './portfolio/portfolio-dashboard/portfolio-dashboard.component';
import { PortfolioItemsComponent } from './portfolio/portfolio-items/portfolio-items.component';
import { PortfolioResourcesComponent } from './portfolio/portfolio-resources/portfolio-resources.component';
import { PortfolioContentComponent } from './portfolio/portfolio-content/portfolio-content.component';
import { PortfolioRegistryComponent } from './portfolio/portfolio-registry/portfolio-registry.component';
import { PortfolioAnalyticsComponent } from './portfolio/portfolio-analytics/portfolio-analytics.component';
import { PortfolioQueryComponent } from './portfolio/portfolio-query/portfolio-query.component';
import { PortfolioDetailComponent } from './portfolio/portfolio-detail/portfolio-detail.component';
import { PortfolioBinderComponent } from './portfolio/portfolio-binder/portfolio-binder.component';
import { PortfolioCollaborationComponent } from './portfolio/portfolio-collaboration/portfolio-collaboration.component';
import { PortfolioItembookComponent } from './portfolio/portfolio-itembook/portfolio-itembook.component';
import { ItembookCharterComponent } from './portfolio/portfolio-itembook/itembook-charter/itembook-charter.component';
import { ItembookWorkspaceComponent } from './portfolio/portfolio-itembook/itembook-workspace/itembook-workspace.component';
import { ItembookCatalogueComponent } from './portfolio/portfolio-itembook/itembook-catalogue/itembook-catalogue.component';
import { ItembookScheduleComponent } from './portfolio/portfolio-itembook/itembook-schedule/itembook-schedule.component';
import { ItembookMetricsComponent } from './portfolio/portfolio-itembook/itembook-metrics/itembook-metrics.component';
import { ItembookLibraryComponent } from './portfolio/portfolio-itembook/itembook-library/itembook-library.component';
import { ItembookLogsComponent } from './portfolio/portfolio-itembook/itembook-logs/itembook-logs.component';
import { PortfolioFolderComponent } from './portfolio/portfolio-folder/portfolio-folder.component';
import { PortfolioGraphComponent } from './portfolio/portfolio-graph/portfolio-graph.component';
import { PortfolioNotebookComponent } from './portfolio/portfolio-notebook/portfolio-notebook.component';
import { PortfolioGuidebookComponent } from './portfolio/portfolio-guidebook/portfolio-guidebook.component';
import { PortfolioSubportfolioComponent } from './portfolio/portfolio-subportfolio/portfolio-subportfolio.component';
import { PortfolioWorkspaceComponent } from './portfolio/portfolio-workspace/portfolio-workspace.component';
import { OfficeComponent } from './office/office.component';
import { OfficeOverviewComponent } from './office/office-overview/office-overview.component';
import { OfficeInboxComponent } from './office/office-inbox/office-inbox.component';
import { OfficeScheduleComponent } from './office/office-schedule/office-schedule.component';
import { OfficeMeetingsComponent } from './office/office-meetings/office-meetings.component';
import { OfficeStudioComponent } from './office/office-studio/office-studio.component';
import { OfficeStudioOverviewComponent } from './office/office-studio/studio-overview/studio-overview.component';
import { OfficeStudioIdeasComponent } from './office/office-studio/studio-ideas/studio-ideas.component';
import { OfficeStudioConceptsComponent } from './office/office-studio/studio-concepts/studio-concepts.component';
import { OfficeStudioDesignsComponent } from './office/office-studio/studio-designs/studio-designs.component';
import { OfficeStudioBlueprintsComponent } from './office/office-studio/studio-blueprints/studio-blueprints.component';
import { OfficeStudioMockupsComponent } from './office/office-studio/studio-mockups/studio-mockups.component';
import { OfficeStudioPrototypesComponent } from './office/office-studio/studio-prototypes/studio-prototypes.component';
import { OfficeStudioTestingComponent } from './office/office-studio/studio-testing/studio-testing.component';
import { OfficeStudioNotesComponent } from './office/office-studio/studio-notes/studio-notes.component';
import { OfficeStudioDocsComponent } from './office/office-studio/studio-docs/studio-docs.component';
import { OfficeStudioContentComponent } from './office/office-studio/studio-content/studio-content.component';
import { OfficeContactsComponent } from './office/office-contacts/office-contacts.component';
import { WalletComponent } from './wallet/wallet.component';
import { WalletDashboardComponent } from './wallet/wallet-dashboard/wallet-dashboard.component';
import { WalletWalletsComponent } from './wallet/wallet-wallets/wallet-wallets.component';
import { WalletsOverviewComponent } from './wallet/wallet-wallets/wallets-overview/wallets-overview.component';
import { WalletsTaxesComponent } from './wallet/wallet-wallets/wallets-taxes/wallets-taxes.component';
import { WalletBankingComponent } from './wallet/wallet-banking/wallet-banking.component';
import { WalletLedgerComponent } from './wallet/wallet-ledger/wallet-ledger.component';
import { WalletEscrowComponent } from './wallet/wallet-escrow/wallet-escrow.component';
import { WalletInvoicesComponent } from './wallet/wallet-invoices/wallet-invoices.component';
import { WalletInvestmentsComponent } from './wallet/wallet-investments/wallet-investments.component';
import { WalletFundingComponent } from './wallet/wallet-funding/wallet-funding.component';
import { WalletBenefitsComponent } from './wallet/wallet-benefits/wallet-benefits.component';
import { WalletGrantsComponent } from './wallet/wallet-grants/wallet-grants.component';
import { WalletGroupEconomicsComponent } from './wallet/wallet-group-economics/wallet-group-economics.component';
import { WalletCampaignsComponent } from './wallet/wallet-campaigns/wallet-campaigns.component';
import { WalletDebtsComponent } from './wallet/wallet-debts/wallet-debts.component';
import { WalletTaxesComponent } from './wallet/wallet-taxes/wallet-taxes.component';
import { SpacesComponent } from './spaces/spaces.component';
import { SpacesDashboardComponent } from './spaces/spaces-dashboard/spaces-dashboard.component';
import { SpacesEventsComponent } from './spaces/spaces-events/spaces-events.component';
import { SpacesChannelsComponent } from './spaces/spaces-channels/spaces-channels.component';
import { SpacesSpacesComponent } from './spaces/spaces-spaces/spaces-spaces.component';
import { SpacesRoomsComponent } from './spaces/spaces-rooms/spaces-rooms.component';
import { SpacesFeedComponent } from './spaces/spaces-feed/spaces-feed.component';
import { SpacesTimelineComponent } from './spaces/spaces-timeline/spaces-timeline.component';
import { SpacesNetworkComponent } from './spaces/spaces-network/spaces-network.component';
import { SpacesNetworkLinknetComponent } from './spaces/spaces-network/network-linknet/network-linknet.component';
import { SpacesNetworkLinktreeComponent } from './spaces/spaces-network/network-linktree/network-linktree.component';
import { SpacesNetworkLinktreeEditorComponent } from './spaces/spaces-network/network-linktree-editor/network-linktree-editor.component';
import { SpacesNetworkLinkforestComponent } from './spaces/spaces-network/network-linkforest/network-linkforest.component';
import { MarketplaceComponent } from './marketplace/marketplace.component';
import { MarketplaceDashboardComponent } from './marketplace/marketplace-dashboard/marketplace-dashboard.component';
import { MarketplaceBarterComponent } from './marketplace/marketplace-barter/marketplace-barter.component';
import { MarketplaceBarterExchangeComponent } from './marketplace/marketplace-barter/marketplace-barter-exchange/marketplace-barter-exchange.component';
import { MarketplaceBarterDealsComponent } from './marketplace/marketplace-barter/marketplace-barter-deals/marketplace-barter-deals.component';
import { MarketplaceBarterOffersComponent } from './marketplace/marketplace-barter/marketplace-barter-offers/marketplace-barter-offers.component';
import { MarketplaceBarterBidsComponent } from './marketplace/marketplace-barter/marketplace-barter-bids/marketplace-barter-bids.component';
import { MarketplaceMarketComponent } from './marketplace/marketplace-market/marketplace-market.component';
import { MarketplaceMarketOverviewComponent } from './marketplace/marketplace-market/marketplace-market-overview/marketplace-market-overview.component';
import { MarketplaceMarketBrowseComponent } from './marketplace/marketplace-market/marketplace-market-browse/marketplace-market-browse.component';
import { MarketplaceMarketCrmComponent } from './marketplace/marketplace-market/marketplace-market-crm/marketplace-market-crm.component';
import { MarketplaceMarketGrantsComponent } from './marketplace/marketplace-market/marketplace-market-grants/marketplace-market-grants.component';
import { MarketplaceMarketLaborComponent } from './marketplace/marketplace-market/marketplace-market-labor/marketplace-market-labor.component';
import { MarketplaceListingsComponent } from './marketplace/marketplace-listings/marketplace-listings.component';
import { MarketplaceListingsCatalogComponent } from './marketplace/marketplace-listings/marketplace-listings-catalog/marketplace-listings-catalog.component';
import { MarketplaceListingsDetailComponent } from './marketplace/marketplace-listings/marketplace-listings-detail/marketplace-listings-detail.component';
import { MarketplaceListingsMineComponent } from './marketplace/marketplace-listings/marketplace-listings-mine/marketplace-listings-mine.component';
import { MarketplaceCampaignsComponent } from './marketplace/marketplace-campaigns/marketplace-campaigns.component';
import { MarketplaceCampaignsOverviewComponent } from './marketplace/marketplace-campaigns/marketplace-campaigns-overview/marketplace-campaigns-overview.component';
import { MarketplaceCampaignsBuilderComponent } from './marketplace/marketplace-campaigns/marketplace-campaigns-builder/marketplace-campaigns-builder.component';
import { MarketplaceCampaignsDiscoverComponent } from './marketplace/marketplace-campaigns/marketplace-campaigns-discover/marketplace-campaigns-discover.component';
import { MarketplaceCampaignsCollectiveComponent } from './marketplace/marketplace-campaigns/marketplace-campaigns-collective/marketplace-campaigns-collective.component';
import { MarketplaceCampaignsCapitalComponent } from './marketplace/marketplace-campaigns/marketplace-campaigns-capital/marketplace-campaigns-capital.component';
import { MarketplaceCampaignsPortfolioComponent } from './marketplace/marketplace-campaigns/marketplace-campaigns-portfolio/marketplace-campaigns-portfolio.component';
import { MarketplaceExchangeComponent } from './marketplace/marketplace-exchange/marketplace-exchange.component';
import { MarketplaceExchangeOverviewComponent } from './marketplace/marketplace-exchange/marketplace-exchange-overview/marketplace-exchange-overview.component';
import { MarketplaceExchangeWalletComponent } from './marketplace/marketplace-exchange/marketplace-exchange-wallet/marketplace-exchange-wallet.component';
import { MarketplaceExchangeBidsComponent } from './marketplace/marketplace-exchange/marketplace-exchange-bids/marketplace-exchange-bids.component';
import { MarketplaceExchangeLaborComponent } from './marketplace/marketplace-exchange/marketplace-exchange-labor/marketplace-exchange-labor.component';
import { MarketplaceExchangeDealRoomComponent } from './marketplace/marketplace-exchange/marketplace-exchange-deal-room/marketplace-exchange-deal-room.component';
import { MarketplaceExchangeCapitalComponent } from './marketplace/marketplace-exchange/marketplace-exchange-capital/marketplace-exchange-capital.component';
import { MarketplaceExchangeResourcesComponent } from './marketplace/marketplace-exchange/marketplace-exchange-resources/marketplace-exchange-resources.component';
import { MarketplaceExchangeCommoditiesComponent } from './marketplace/marketplace-exchange/marketplace-exchange-commodities/marketplace-exchange-commodities.component';
import { MarketplaceExchangeAssetTransferComponent } from './marketplace/marketplace-exchange/marketplace-exchange-asset-transfer/marketplace-exchange-asset-transfer.component';
import { MarketplaceExchangeBarterComponent } from './marketplace/marketplace-exchange/marketplace-exchange-barter/marketplace-exchange-barter.component';
import { MarketplaceEscrowComponent } from './marketplace/marketplace-escrow/marketplace-escrow.component';
import { MarketplaceEscrowOverviewComponent } from './marketplace/marketplace-escrow/marketplace-escrow-overview/marketplace-escrow-overview.component';
import { MarketplaceEscrowBankComponent } from './marketplace/marketplace-escrow/marketplace-escrow-bank/marketplace-escrow-bank.component';
import { HubComponent } from './hub/hub.component';
import { HubDashboardComponent } from './hub/hub-dashboard/hub-dashboard.component';
import { HubGovernanceComponent } from './hub/hub-governance/hub-governance.component';
import { HubVotingComponent } from './hub/hub-voting/hub-voting.component';
import { HubAllocationComponent } from './hub/hub-allocation/hub-allocation.component';
import { HubDistributionComponent } from './hub/hub-distribution/hub-distribution.component';
import { HubCollaborationComponent } from './hub/hub-collaboration/hub-collaboration.component';
import { HubRestitutionComponent } from './hub/hub-restitution/hub-restitution.component';
import { HubNegotiationsComponent } from './hub/hub-negotiations/hub-negotiations.component';
import { HubTeamsComponent } from './hub/hub-teams/hub-teams.component';
import { HubOrganizationsComponent } from './hub/hub-organizations/hub-organizations.component';
import { HubCollectivesComponent } from './hub/hub-collectives/hub-collectives.component';
import { HubCooperativesComponent } from './hub/hub-cooperatives/hub-cooperatives.component';
import { HubFederationsComponent } from './hub/hub-federations/hub-federations.component';
import { HubAutonomousComponent } from './hub/hub-autonomous/hub-autonomous.component';
import { HubOpenSourceComponent } from './hub/hub-open-source/hub-open-source.component';
import { HubGroupEconomicsComponent } from './hub/hub-group-economics/hub-group-economics.component';
import { HubResourceCrowdfundComponent } from './hub/hub-resource-crowdfund/hub-resource-crowdfund.component';
import { HubCommunityShowcaseComponent } from './hub/hub-community-showcase/hub-community-showcase.component';
import { AssistantComponent } from './assistant/assistant.component';
import { TestComponent } from './index/test/test';

export const routes: Routes = [
    {path: '', redirectTo: 'home', pathMatch: 'full'},
    {
        path: 'home',
        component: HomeComponent,
        title: 'Kogi \u2014 Home',
        children: [
            {path: '', component: HomeLandingComponent, title: 'Kogi \u2014 Home'},
            {path: 'about', component: HomeAboutComponent, title: 'Kogi \u2014 About'},
            {path: 'platform', component: HomePlatformComponent, title: 'Kogi \u2014 Platform'},
            {path: 'ecosystem', component: HomeEcosystemComponent, title: 'Kogi \u2014 Ecosystem'},
            {path: '**', redirectTo: ''}
        ]
    },
    {path: 'login', component: LoginComponent, title: 'Kogi \u2014 Create Account'},
    {path: 'registration', component: RegistrationComponent, title: 'Registration'},
    {path: 'onboarding', component: OnboardingComponent, title: 'Onboarding'},
    {path: 'creating-workspace', component: CreatingWorkspaceComponent, title: 'Creating Workspace'},
    {
        path: 'dashboard',
        component: DashboardComponent, 
        title: 'Dashboard',
        children: [
            {path: '', redirectTo: 'overview', pathMatch: 'full'},
            {path: 'calendar', component: DashboardCalendarComponent, title: 'Dashboard Calendar'},
            {path: 'overview', component: DashboardOverviewComponent, title: 'Dashboard Overview'},
        ]

    },
    {path: 'portfolio/new', component: PortfolioCreateComponent, title: 'Create Portfolio'},
    {
        path: 'portfolio',
        component: PortfolioComponent,
        title: 'Portfolio',
        children: [
            {path: '', redirectTo: 'dashboard', pathMatch: 'full'},
            {path: 'dashboard', component: PortfolioDashboardComponent, title: 'Portfolio Dashboard'},
            {path: 'items', component: PortfolioItemsComponent, title: 'Portfolio Items'},
            {path: 'resources', component: PortfolioResourcesComponent, title: 'Portfolio Resources'},
            {path: 'content', component: PortfolioContentComponent, title: 'Portfolio Content'},
            {path: 'registry', component: PortfolioRegistryComponent, title: 'Portfolio Registry'},
            {path: 'analytics', component: PortfolioAnalyticsComponent, title: 'Portfolio Analytics'},
            {path: 'query', component: PortfolioQueryComponent, title: 'Portfolio Query'},
            {path: 'detail/:kind', component: PortfolioDetailComponent, title: 'Portfolio Detail'},
            {path: 'binder', component: PortfolioBinderComponent, title: 'Portfolio Binder'},
            {path: 'collaboration', component: PortfolioCollaborationComponent, title: 'Portfolio Collaboration'},
            {
                path: 'itembook',
                component: PortfolioItembookComponent,
                children: [
                    {path: '', redirectTo: 'charter', pathMatch: 'full'},
                    {path: 'charter', component: ItembookCharterComponent, title: 'ItemBook Charter'},
                    {path: 'workspace', component: ItembookWorkspaceComponent, title: 'ItemBook Workspace'},
                    {path: 'catalogue', component: ItembookCatalogueComponent, title: 'ItemBook Catalogue'},
                    {path: 'schedule', component: ItembookScheduleComponent, title: 'ItemBook Schedule'},
                    {path: 'metrics', component: ItembookMetricsComponent, title: 'ItemBook Metrics'},
                    {path: 'library', component: ItembookLibraryComponent, title: 'ItemBook Library'},
                    {path: 'logs', component: ItembookLogsComponent, title: 'ItemBook Logs'},
                    {path: '**', redirectTo: 'charter'}
                ]
            },
            {path: 'folder', component: PortfolioFolderComponent, title: 'Folder View'},
            {path: 'graph', component: PortfolioGraphComponent, title: 'Graph View'},
            {path: 'notebook', component: PortfolioNotebookComponent, title: 'Notebook'},
            {path: 'guidebook', component: PortfolioGuidebookComponent, title: 'Guidebook'},
            {path: 'subportfolio', component: PortfolioSubportfolioComponent, title: 'Subportfolio'},
            {path: 'workspace', component: PortfolioWorkspaceComponent, title: 'Portfolio Workspace'},
            {path: '**', redirectTo: 'dashboard'}
        ]
    },
    {
        path: 'office',
        component: OfficeComponent,
        title: 'Office',
        children: [
            {path: '', redirectTo: 'overview', pathMatch: 'full'},
            {path: 'dashboard', redirectTo: 'overview', pathMatch: 'full'},
            {path: 'overview', component: OfficeOverviewComponent, title: 'Office Overview'},
            {path: 'inbox', component: OfficeInboxComponent, title: 'Office Inbox'},
            {path: 'schedule', component: OfficeScheduleComponent, title: 'Office Schedule'},
            {path: 'meetings', component: OfficeMeetingsComponent, title: 'Office Meetings'},
            {path: 'contacts', component: OfficeContactsComponent, title: 'Office Contacts'},
            {
                path: 'studio',
                component: OfficeStudioComponent,
                children: [
                    {path: '', redirectTo: 'overview', pathMatch: 'full'},
                    {path: 'overview', component: OfficeStudioOverviewComponent, title: 'Studio Overview'},
                    {path: 'ideas', component: OfficeStudioIdeasComponent, title: 'Studio Ideas'},
                    {path: 'concepts', component: OfficeStudioConceptsComponent, title: 'Studio Concepts'},
                    {path: 'designs', component: OfficeStudioDesignsComponent, title: 'Studio Designs'},
                    {path: 'blueprints', component: OfficeStudioBlueprintsComponent, title: 'Studio Blueprints'},
                    {path: 'mockups', component: OfficeStudioMockupsComponent, title: 'Studio Mockups'},
                    {path: 'prototypes', component: OfficeStudioPrototypesComponent, title: 'Studio Prototypes'},
                    {path: 'testing', component: OfficeStudioTestingComponent, title: 'Studio Testing'},
                    {path: 'notes', component: OfficeStudioNotesComponent, title: 'Studio Notes'},
                    {path: 'docs', component: OfficeStudioDocsComponent, title: 'Studio Docs'},
                    {path: 'content', component: OfficeStudioContentComponent, title: 'Studio Content'},
                    {path: '**', redirectTo: 'overview'}
                ]
            },
            {path: '**', redirectTo: 'overview'}
        ]
    },
    {
        path: 'wallet',
        component: WalletComponent,
        title: 'Wallet',
        children: [
            {path: '', redirectTo: 'dashboard', pathMatch: 'full'},
            {path: 'dashboard', component: WalletDashboardComponent, title: 'Wallet Dashboard'},
            {path: 'banking', component: WalletBankingComponent, title: 'Wallet Banking'},
            {path: 'ledger', component: WalletLedgerComponent, title: 'Wallet Ledger'},
            {path: 'escrow', component: WalletEscrowComponent, title: 'Wallet Escrow'},
            {path: 'invoices', component: WalletInvoicesComponent, title: 'Wallet Invoices'},
            {path: 'investments', component: WalletInvestmentsComponent, title: 'Wallet Investments'},
            {path: 'funding', component: WalletFundingComponent, title: 'Wallet Funding'},
            {path: 'benefits', component: WalletBenefitsComponent, title: 'Wallet Benefits'},
            {path: 'grants', component: WalletGrantsComponent, title: 'Wallet Grants'},
            {path: 'group-economics', component: WalletGroupEconomicsComponent, title: 'Wallet Group Economics'},
            {path: 'campaigns', component: WalletCampaignsComponent, title: 'Wallet Campaigns'},
            {path: 'debts', component: WalletDebtsComponent, title: 'Wallet Debts'},
            {path: 'taxes', component: WalletTaxesComponent, title: 'Wallet Taxes'},
            {path: 'accounts', redirectTo: 'ledger', pathMatch: 'full'},
            {path: 'walelts', redirectTo: 'wallets', pathMatch: 'full'},
            {
                path: 'wallets',
                component: WalletWalletsComponent,
                children: [
                    {path: '', redirectTo: 'overview', pathMatch: 'full'},
                    {path: 'overview', component: WalletsOverviewComponent, title: 'Wallet Overview'},
                    {path: 'taxes', component: WalletsTaxesComponent, title: 'Wallet Taxes'},
                    {path: '**', redirectTo: 'overview'}
                ]
            },
            {path: '**', redirectTo: 'dashboard'}
        ]
    },
    {
        path: 'marketplace',
        component: MarketplaceComponent,
        title: 'Marketplace',
        children: [
            {path: '', redirectTo: 'dashboard', pathMatch: 'full'},
            {path: 'dashboard', component: MarketplaceDashboardComponent, title: 'Marketplace Dashboard'},
            {
                path: 'market',
                component: MarketplaceMarketComponent,
                children: [
                    {path: '', redirectTo: 'overview', pathMatch: 'full'},
                    {path: 'overview', component: MarketplaceMarketOverviewComponent, title: 'Marketplace Overview'},
                    {path: 'browse', component: MarketplaceMarketBrowseComponent, title: 'Marketplace Browse'},
                    {path: 'crm', component: MarketplaceMarketCrmComponent, title: 'Marketplace CRM'},
                    {path: 'grants', component: MarketplaceMarketGrantsComponent, title: 'Marketplace Grants'},
                    {path: 'labor', component: MarketplaceMarketLaborComponent, title: 'Labor Market'},
                    {path: '**', redirectTo: 'overview'}
                ]
            },
            {
                path: 'listings',
                component: MarketplaceListingsComponent,
                children: [
                    {path: '', redirectTo: 'catalog', pathMatch: 'full'},
                    {path: 'catalog', component: MarketplaceListingsCatalogComponent, title: 'Marketplace Listings'},
                    {path: 'detail', component: MarketplaceListingsDetailComponent, title: 'Listing Detail'},
                    {path: 'my-listings', component: MarketplaceListingsMineComponent, title: 'My Listings'},
                    {path: '**', redirectTo: 'catalog'}
                ]
            },
            {
                path: 'exchange',
                component: MarketplaceExchangeComponent,
                children: [
                    {path: '', redirectTo: 'overview', pathMatch: 'full'},
                    {path: 'overview', component: MarketplaceExchangeOverviewComponent, title: 'Exchange Overview'},
                    {path: 'wallet', component: MarketplaceExchangeWalletComponent, title: 'Exchange Wallet'},
                    {path: 'bids', component: MarketplaceExchangeBidsComponent, title: 'Bids & Offers'},
                    {path: 'labor', component: MarketplaceExchangeLaborComponent, title: 'Labor Market'},
                    {path: 'deal-room', component: MarketplaceExchangeDealRoomComponent, title: 'Deal Room'},
                    {path: 'capital', component: MarketplaceExchangeCapitalComponent, title: 'Capital Exchange'},
                    {path: 'resources', component: MarketplaceExchangeResourcesComponent, title: 'Resource Exchange'},
                    {path: 'commodities', component: MarketplaceExchangeCommoditiesComponent, title: 'Commodities'},
                    {path: 'asset-transfer', component: MarketplaceExchangeAssetTransferComponent, title: 'Asset Transfer'},
                    {path: 'barter', component: MarketplaceExchangeBarterComponent, title: 'Barter Exchange'},
                    {path: '**', redirectTo: 'overview'}
                ]
            },
            {
                path: 'barter',
                component: MarketplaceBarterComponent,
                children: [
                    {path: '', redirectTo: 'exchange', pathMatch: 'full'},
                    {path: 'exchange', component: MarketplaceBarterExchangeComponent, title: 'Barter Exchange'},
                    {path: 'deals', component: MarketplaceBarterDealsComponent, title: 'Deal Room'},
                    {path: 'offers', component: MarketplaceBarterOffersComponent, title: 'Offers'},
                    {path: 'bids', component: MarketplaceBarterBidsComponent, title: 'Bids'},
                    {path: '**', redirectTo: 'exchange'}
                ]
            },
            {
                path: 'escrow',
                component: MarketplaceEscrowComponent,
                children: [
                    {path: '', redirectTo: 'overview', pathMatch: 'full'},
                    {path: 'overview', component: MarketplaceEscrowOverviewComponent, title: 'Escrow Overview'},
                    {path: 'bank', component: MarketplaceEscrowBankComponent, title: 'Bank Escrow'},
                    {path: '**', redirectTo: 'overview'}
                ]
            },
            {
                path: 'campaigns',
                component: MarketplaceCampaignsComponent,
                children: [
                    {path: '', redirectTo: 'overview', pathMatch: 'full'},
                    {path: 'overview', component: MarketplaceCampaignsOverviewComponent, title: 'Campaigns Overview'},
                    {path: 'builder', component: MarketplaceCampaignsBuilderComponent, title: 'Campaign Builder'},
                    {path: 'discover', component: MarketplaceCampaignsDiscoverComponent, title: 'Campaign Discover'},
                    {path: 'collective', component: MarketplaceCampaignsCollectiveComponent, title: 'Collective Campaign'},
                    {path: 'capital', component: MarketplaceCampaignsCapitalComponent, title: 'Campaigns & Capital'},
                    {path: 'portfolio', component: MarketplaceCampaignsPortfolioComponent, title: 'Campaigns Portfolio'},
                    {path: '**', redirectTo: 'overview'}
                ]
            },
            {path: '**', redirectTo: 'dashboard'}
        ]
    },
    {
        path: 'hub',
        component: HubComponent,
        title: 'Organization Hub',
        children: [
            {path: '', redirectTo: 'dashboard', pathMatch: 'full'},
            {path: 'dashboard', component: HubDashboardComponent, title: 'Hub Dashboard'},
            {path: 'governance', component: HubGovernanceComponent, title: 'Hub Governance'},
            {path: 'voting', component: HubVotingComponent, title: 'Hub Voting'},
            {path: 'allocation', component: HubAllocationComponent, title: 'Hub Allocation'},
            {path: 'distribution', component: HubDistributionComponent, title: 'Hub Distribution'},
            {path: 'collaboration', component: HubCollaborationComponent, title: 'Hub Collaboration'},
            {path: 'restitution', component: HubRestitutionComponent, title: 'Hub Restitution'},
            {path: 'negotiations', component: HubNegotiationsComponent, title: 'Hub Negotiations'},
            {path: 'teams', component: HubTeamsComponent, title: 'Hub Teams'},
            {path: 'organizations', component: HubOrganizationsComponent, title: 'Hub Organizations'},
            {path: 'collectives', component: HubCollectivesComponent, title: 'Hub Collectives'},
            {path: 'cooperatives', component: HubCooperativesComponent, title: 'Hub Cooperatives'},
            {path: 'federations', component: HubFederationsComponent, title: 'Hub Federations'},
            {path: 'autonomous', component: HubAutonomousComponent, title: 'Hub Autonomous Cells'},
            {path: 'open-source', component: HubOpenSourceComponent, title: 'Hub Open Source'},
            {path: 'group-economics', component: HubGroupEconomicsComponent, title: 'Hub Group Economics'},
            {path: 'resource-crowdfund', component: HubResourceCrowdfundComponent, title: 'Hub Resource Crowdfund'},
            {path: 'community-showcase', component: HubCommunityShowcaseComponent, title: 'Hub Community Showcase'},
            {path: '**', redirectTo: 'dashboard'}
        ]
    },
    {path: 'assistant', component: AssistantComponent, title: 'Assistant'},
    {
        path: 'spaces',
        component: SpacesComponent,
        title: 'Spaces',
        children: [
            {path: '', redirectTo: 'dashboard', pathMatch: 'full'},
            {path: 'overview', redirectTo: 'dashboard', pathMatch: 'full'},
            {path: 'dashboard', component: SpacesDashboardComponent, title: 'Spaces Dashboard'},
            {path: 'spaces', component: SpacesSpacesComponent, title: 'Spaces'},
            {path: 'rooms', component: SpacesRoomsComponent, title: 'Rooms'},
            {path: 'feed', component: SpacesFeedComponent, title: 'Spaces Feed'},
            {path: 'timeline', component: SpacesTimelineComponent, title: 'Spaces Timeline'},
            {path: 'events', component: SpacesEventsComponent, title: 'Spaces Events'},
            {
                path: 'network',
                component: SpacesNetworkComponent,
                children: [
                    {path: '', redirectTo: 'overview', pathMatch: 'full'},
                    {path: 'overview', component: SpacesNetworkLinknetComponent, title: 'Linknetwork Overview'},
                    {path: 'linktree/editor', component: SpacesNetworkLinktreeEditorComponent, title: 'Linktree Editor'},
                    {path: 'linktree', component: SpacesNetworkLinktreeComponent, title: 'Linktree'},
                    {path: 'linkforest', component: SpacesNetworkLinkforestComponent, title: 'Linkforest'},
                    {path: '**', redirectTo: 'overview'}
                ]
            },
            {path: 'channels', component: SpacesChannelsComponent, title: 'Spaces Channels'}
        ]
    },
    {path: 'test', component: TestComponent, title: 'Testing Page'},
    {path: '**', redirectTo: '' }
];
