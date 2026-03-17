import { NgClass } from '@angular/common';
import { Component, OnDestroy, OnInit, inject } from '@angular/core';
import { ActivatedRoute, NavigationEnd, Router, RouterLink, RouterOutlet } from '@angular/router';
import { Subscription } from 'rxjs';

@Component({
  selector: 'app-wallet',
  standalone: true,
  imports: [NgClass, RouterLink, RouterOutlet],
  templateUrl: './wallet.component.html',
  styleUrl: './wallet.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class WalletComponent implements OnInit, OnDestroy {
  activeSecondaryNav: WalletSecondaryNav = 'wallets';

  readonly activeSecondaryNavClass = 'text-[#10b981]';
  readonly inactiveSecondaryNavClass = 'text-[#5d747c] hover:text-[#e6f1f4]';
  readonly secondaryNavBaseClass = 'cursor-pointer transition-colors';

  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);
  private navSub?: Subscription;

  ngOnInit(): void {
    this.syncSecondaryNav();
    this.navSub = this.router.events.subscribe((event) => {
      if (event instanceof NavigationEnd) {
        this.syncSecondaryNav();
      }
    });
  }

  ngOnDestroy(): void {
    this.navSub?.unsubscribe();
  }

  setActiveSecondaryNav(view: WalletSecondaryNav): void {
    this.activeSecondaryNav = view;

    if (view === 'dashboard') {
      void this.router.navigate(['/wallet/dashboard']);
      return;
    }

    if (view === 'wallets') {
      void this.router.navigate(['/wallet']);
    }
  }

  secondaryNavItemClass(view: WalletSecondaryNav): string {
    const stateClass = this.activeSecondaryNav === view ? this.activeSecondaryNavClass : this.inactiveSecondaryNavClass;
    return `${this.secondaryNavBaseClass} ${stateClass}`;
  }

  private syncSecondaryNav(): void {
    const childPath = this.route.firstChild?.snapshot.routeConfig?.path;
    if (childPath === 'dashboard') {
      this.activeSecondaryNav = 'dashboard';
      return;
    }

    this.activeSecondaryNav = 'wallets';
  }
}

type WalletSecondaryNav =
  | 'dashboard'
  | 'wallets'
  | 'accounts'
  | 'portable-benefits'
  | 'grants'
  | 'group-economics';
