import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-escrow-overview',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-escrow-overview.component.html',
  styleUrl: './marketplace-escrow-overview.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceEscrowOverviewComponent {}
