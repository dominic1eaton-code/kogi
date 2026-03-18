import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-market-overview',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-market-overview.component.html',
  styleUrl: './marketplace-market-overview.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceMarketOverviewComponent {}
