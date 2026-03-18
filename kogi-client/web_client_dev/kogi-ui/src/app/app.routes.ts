import { Routes } from '@angular/router';
import { LoginComponent } from './login/login.component';
import { RegistrationComponent } from './registration/registration.component';
import { OnboardingComponent } from './onboarding/onboarding.component';
import { DashboardComponent } from './dashboard/dashboard.component'
import { CreatingWorkspaceComponent } from './creating-workspace/creating-workspace.component';
import { PortfolioComponent } from './portfolio/portfolio.component';
import { PortfolioCreateComponent } from './portfolio/portfolio-create/portfolio-create.component';
import { OfficeComponent } from './office/office.component';
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
import { HubComponent } from './hub/hub.component';
import { AssistantComponent } from './assistant/assistant.component';
import { TestComponent } from './test/test';

export const routes: Routes = [
    {path: '', component: LoginComponent, title: 'Kogi \u2014 Create Account'},
    {path: 'login', component: LoginComponent, title: 'Kogi \u2014 Create Account'},
    {path: 'registration', component: RegistrationComponent, title: 'Registration'},
    {path: 'onboarding', component: OnboardingComponent, title: 'Onboarding'},
    {path: 'creating-workspace', component: CreatingWorkspaceComponent, title: 'Creating Workspace'},
    {path: 'dashboard', component: DashboardComponent, title: 'Dashboard'},
    {path: 'portfolio/new', component: PortfolioCreateComponent, title: 'Create Portfolio'},
    {path: 'portfolio', component: PortfolioComponent, title: 'Portfolio'},
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
    {path: 'office', component: OfficeComponent, title: "Office"},
    {path: 'marketplace', component: MarketplaceComponent, title: "Marketplace"},
    {path: 'hub', component: HubComponent, title: "Organization Hub"},
    {path: 'assistant', component: AssistantComponent, title: "Assistant"},
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
    {path: 'test', component: TestComponent, title: "Testing Page"},
    {path: '**', redirectTo: '' }
];
