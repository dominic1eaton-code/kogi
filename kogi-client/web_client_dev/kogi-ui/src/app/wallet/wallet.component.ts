import { CommonModule } from '@angular/common';
import { Component, inject } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet, Router } from '@angular/router';
import { NavigationComponent } from '../index/navigation/navigation.component';

type WalletNavItem = {
  label: string;
  route: string;
  exact?: boolean;
};

type WalletNavSection = {
  label: string;
  items: WalletNavItem[];
};

@Component({
  selector: 'app-wallet',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet, CommonModule, NavigationComponent],
  templateUrl: './wallet.component.html',
  styleUrl: './wallet.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class WalletComponent {
  showSecondaryNav = true;

  primaryTabs: WalletNavItem[] = [
    { label: 'Dashboard', route: '/wallet/dashboard', exact: true },
    { label: 'Banking', route: '/wallet/banking' },
    { label: 'Ledger', route: '/wallet/ledger' },
    { label: 'Escrow', route: '/wallet/escrow' },
    { label: 'Invoices', route: '/wallet/invoices' },
    { label: 'Investments', route: '/wallet/investments' },
    { label: 'Funding', route: '/wallet/funding' },
    { label: 'Benefits', route: '/wallet/benefits' },
    { label: 'Grants', route: '/wallet/grants' },
    { label: 'Group Economics', route: '/wallet/group-economics' },
    { label: 'Campaigns', route: '/wallet/campaigns' },
    { label: 'Debts', route: '/wallet/debts' },
    { label: 'Taxes', route: '/wallet/taxes' }
  ];

  secondaryNav: WalletNavSection[] = [
    {
      label: 'Core',
      items: [
        { label: 'Overview', route: '/wallet/dashboard', exact: true },
        { label: 'Wallets', route: '/wallet/wallets/overview' },
        { label: 'Banking', route: '/wallet/banking' },
        { label: 'Taxes', route: '/wallet/taxes' }
      ]
    },
    {
      label: 'Transactions',
      items: [
        { label: 'Accounts Ledger', route: '/wallet/ledger' },
        { label: 'Invoices', route: '/wallet/invoices' },
        { label: 'Escrow', route: '/wallet/escrow' },
        { label: 'Campaigns', route: '/wallet/campaigns' },
        { label: 'Debts', route: '/wallet/debts' }
      ]
    },
    {
      label: 'Investments',
      items: [
        { label: 'Investments', route: '/wallet/investments' },
        { label: 'Funding & Equity', route: '/wallet/funding' },
        { label: 'Securities', route: '/wallet/investments' }
      ]
    },
    {
      label: 'Benefits & Grants',
      items: [
        { label: 'Portable Benefits', route: '/wallet/benefits' },
        { label: 'Grants', route: '/wallet/grants' }
      ]
    },
    {
      label: 'Group Economics',
      items: [
        { label: 'Group Treasury', route: '/wallet/group-economics' },
        { label: 'Distributions', route: '/wallet/group-economics' }
      ]
    }
  ];

  private router = inject(Router);

  toggleSecondaryNav(): void {
    this.showSecondaryNav = !this.showSecondaryNav;
  }

  logout(): void {
    console.log('Logging out of client session...');
    this.router.navigate(['/login']);
  }
}
