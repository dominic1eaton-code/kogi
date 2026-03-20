import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletTaxesComponent } from './wallet-taxes.component';

describe('WalletTaxesComponent', () => {
  let component: WalletTaxesComponent;
  let fixture: ComponentFixture<WalletTaxesComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletTaxesComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletTaxesComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
