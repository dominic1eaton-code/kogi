import { Routes } from '@angular/router';
import { LoginComponent } from './index/login/login.component';
import { RegistrationComponent } from './index/registration/registration.component';
import { OnboardingComponent } from './index/onboarding/onboarding.component';
import { CreatingWorkspaceComponent } from './index/creating-workspace/creating-workspace.component';
import { DashboardComponent } from './dashboard/dashboard.component'
import { DashboardOverviewComponent } from './dashboard/dashboard-overview/dashboard-overview.component'
import { DashboardCalendarComponent } from './dashboard/dashboard-calendar/dashboard-calendar.component';
import { PortfolioComponent } from './portfolio/portfolio.component';
import { PortfolioCreateComponent } from './portfolio/portfolio-create/portfolio-create.component';
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
import { HubComponent } from './hub/hub.component';
import { AssistantComponent } from './assistant/assistant.component';
import { TestComponent } from './index/test/test';

export const routes: Routes = [
    {path: '', component: LoginComponent, title: 'Kogi \u2014 Create Account'},
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
    {path: 'portfolio', component: PortfolioComponent, title: 'Portfolio'},
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
            }
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
    {path: 'hub', component: HubComponent, title: 'Organization Hub'},
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
